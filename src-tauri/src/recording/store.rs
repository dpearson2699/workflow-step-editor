//! The `WorkflowStore` persistence seam and its JSON implementation
//! (issue #7, DEC-002).
//!
//! `append_event` is the single compound per-event persistence operation:
//! it writes the three screenshot PNGs under `shots/` (temp file plus
//! rename), then appends one flushed JSONL line. No other module writes
//! workflow data. The claim is process-level consistency, not power-loss
//! durability.
//!
//! Confinement: workflow ids are validated against traversal, every
//! directory in the path must be a real (non-symlink) directory, file
//! opens use `O_NOFOLLOW`, and created folders and files are owner-only.

use std::fs::{self, DirBuilder, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use serde::Deserialize;

use crate::domain::schema::{Event, Manifest, ShotPaths, SCHEMA_VERSION};
use crate::recording::clock::{folder_prefix, manifest_timestamp, Clock};

const MANIFEST_FILE: &str = "workflow.json";
const EVENTS_FILE: &str = "events.jsonl";
const SHOTS_DIR: &str = "shots";
const DIR_MODE: u32 = 0o700;
const FILE_MODE: u32 = 0o600;
const CREATE_ATTEMPTS: u32 = 16;

/// The encoded screenshot triple for one event, as opaque PNG bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShotPayloads {
    pub full: Vec<u8>,
    pub window: Vec<u8>,
    pub element: Vec<u8>,
}

/// One row of `list`, extended with the landing-page presentation data
/// (DEC-006): step count from manifest steps, the first-to-last event
/// span, and the first step's window-crop reference. Damaged event logs
/// degrade to `None` fields instead of hiding the row.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct WorkflowSummary {
    pub id: String,
    pub name: String,
    pub created_at: String,
    /// Number of manifest steps (not raw events).
    pub step_count: u64,
    /// Milliseconds from the first to the last event timestamp; zero with
    /// fewer than two events; `None` when the event log is unreadable.
    pub duration_ms: Option<u64>,
    /// Event id whose window crop is the row thumbnail (the first step's
    /// first event); `None` when there is no step or the event log is
    /// unreadable. The frontend resolves it through the scoped
    /// screenshot read (DEC-007); no path crosses IPC.
    pub thumbnail_event_id: Option<String>,
}

/// The allowlisted screenshot variants of the per-event triple (DEC-007).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShotVariant {
    Full,
    Window,
    Element,
}

impl ShotVariant {
    fn file_suffix(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Window => "window",
            Self::Element => "element",
        }
    }
}

/// Error for a variant name outside the DEC-007 allowlist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidShotVariant(String);

impl std::fmt::Display for InvalidShotVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "unknown screenshot variant: {:?} (expected full, window, or element)",
            self.0,
        )
    }
}

impl std::error::Error for InvalidShotVariant {}

impl std::str::FromStr for ShotVariant {
    type Err = InvalidShotVariant;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "full" => Ok(Self::Full),
            "window" => Ok(Self::Window),
            "element" => Ok(Self::Element),
            other => Err(InvalidShotVariant(other.to_owned())),
        }
    }
}

/// The result of `load`: the manifest plus the raw event log.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct LoadedWorkflow {
    pub manifest: Manifest,
    pub events: Vec<Event>,
}

/// The result of `create`: the initial persisted manifest and the
/// unpublished guard for the new folder.
#[derive(Debug)]
pub struct CreatedWorkflow {
    /// The initially valid empty v1 manifest as persisted.
    pub manifest: Manifest,
    pub guard: UnpublishedWorkflow,
}

/// Store-owned guard for a workflow folder whose id has not been published
/// yet. Dropping it unarmed removes the folder: this is startup rollback,
/// not user-facing workflow deletion. Call [`Self::publish`] once the
/// workflow id leaves the store's caller.
#[derive(Debug)]
pub struct UnpublishedWorkflow {
    id: String,
    path: PathBuf,
    armed: bool,
}

impl UnpublishedWorkflow {
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Disarms the rollback and returns the now-published workflow id.
    pub fn publish(mut self) -> String {
        self.armed = false;
        self.id.clone()
    }
}

impl Drop for UnpublishedWorkflow {
    fn drop(&mut self) {
        if self.armed {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}

/// Errors from the persistence seam.
#[derive(Debug)]
pub enum StoreError {
    InvalidWorkflowId(String),
    InvalidEventId(String),
    /// The event's recorded shot paths are not the canonical paths for its
    /// id, so the line would not point at the files the store writes.
    InvalidShotPaths { event_id: String },
    NotFound(String),
    SymlinkRejected(PathBuf),
    /// The manifest's own `id` does not name the workflow folder the save
    /// targets, so writing it would overwrite one workflow with another's
    /// identity and steps.
    ManifestIdMismatch {
        workflow_id: String,
        manifest_id: String,
    },
    UnsupportedSchemaVersion(u64),
    CorruptManifest { workflow_id: String, detail: String },
    CorruptEvents { workflow_id: String, line: usize, detail: String },
    Io { context: String, source: std::io::Error },
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidWorkflowId(id) => write!(f, "invalid workflow id: {id:?}"),
            Self::InvalidEventId(id) => write!(f, "invalid event id: {id:?}"),
            Self::InvalidShotPaths { event_id } => {
                write!(f, "event {event_id} does not record its canonical shot paths")
            }
            Self::NotFound(id) => write!(f, "workflow not found: {id}"),
            Self::SymlinkRejected(path) => {
                write!(f, "refusing symlinked store path: {}", path.display())
            }
            Self::ManifestIdMismatch {
                workflow_id,
                manifest_id,
            } => {
                write!(
                    f,
                    "manifest id {manifest_id:?} does not match workflow {workflow_id:?}",
                )
            }
            Self::UnsupportedSchemaVersion(version) => {
                write!(
                    f,
                    "unsupported schema version {version}; this build supports version {SCHEMA_VERSION}",
                )
            }
            Self::CorruptManifest { workflow_id, detail } => {
                write!(f, "corrupt manifest for workflow {workflow_id}: {detail}")
            }
            Self::CorruptEvents { workflow_id, line, detail } => {
                write!(
                    f,
                    "corrupt event log for workflow {workflow_id} at line {line}: {detail}",
                )
            }
            Self::Io { context, source } => write!(f, "{context}: {source}"),
        }
    }
}

impl std::error::Error for StoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

/// The persistence seam. All workflow data goes through this trait; the
/// JSON implementation is [`JsonWorkflowStore`].
pub trait WorkflowStore: Send + Sync {
    /// Creates the readable per-workflow folder, `shots/`, an empty event
    /// log, and an initially valid empty v1 manifest, so interrupted
    /// recordings stay listable. Returns the manifest and the unpublished
    /// guard.
    fn create(&self, name: &str) -> Result<CreatedWorkflow, StoreError>;

    /// The compound per-event persistence operation: writes each PNG of
    /// the triple to a temporary file and renames it into place under
    /// `shots/`, then appends and flushes one complete JSONL line. On any
    /// shot-write failure no JSONL line is appended.
    fn append_event(
        &self,
        workflow_id: &str,
        event: &Event,
        shots: &ShotPayloads,
    ) -> Result<(), StoreError>;

    /// Loads the manifest and the event log. A torn final JSONL line
    /// (no trailing newline, incomplete JSON) is skipped; a corrupt
    /// newline-terminated line is an error. A manifest whose
    /// `schema_version` is not 1 is an explicit error, and so is a
    /// missing event log: `create` always writes one.
    fn load(&self, workflow_id: &str) -> Result<LoadedWorkflow, StoreError>;

    /// Atomically replaces `workflow.json` (temp file plus rename). The
    /// manifest must carry the target workflow's own `id` and schema
    /// version 1; a mismatch is rejected before any write.
    fn save_manifest(&self, workflow_id: &str, manifest: &Manifest) -> Result<(), StoreError>;

    /// Lists readable workflows newest first (descending id: ids carry
    /// the creation-timestamp prefix). Folders whose manifest fails to
    /// load are skipped so one damaged folder cannot hide the rest; a
    /// damaged event log only degrades that row's summary fields.
    fn list(&self) -> Result<Vec<WorkflowSummary>, StoreError>;

    /// Resolves and validates the workflow folder (id shape, real
    /// non-symlink directories) and returns its absolute path for
    /// backend-side use such as the Finder reveal. The path never
    /// crosses IPC (DEC-007).
    fn locate(&self, workflow_id: &str) -> Result<PathBuf, StoreError>;

    /// Reads one canonical screenshot of an event's triple as raw PNG
    /// bytes. Ids are validated, the path is derived (never accepted),
    /// and symlinks are refused, so the read stays confined to the
    /// workflow's `shots/` folder (DEC-007).
    fn read_shot(
        &self,
        workflow_id: &str,
        event_id: &str,
        variant: ShotVariant,
    ) -> Result<Vec<u8>, StoreError>;
}

/// JSON filesystem implementation of [`WorkflowStore`].
///
/// Takes its root directory and wall-clock source as explicit parameters;
/// it never reads ambient configuration. The constructor performs no IO —
/// the root directory is created on first `create`.
pub struct JsonWorkflowStore {
    root: PathBuf,
    clock: Arc<dyn Clock>,
    temp_counter: AtomicU64,
}

impl JsonWorkflowStore {
    pub fn new(root: PathBuf, clock: Arc<dyn Clock>) -> Self {
        Self {
            root,
            clock,
            temp_counter: AtomicU64::new(0),
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn workflow_dir(&self, workflow_id: &str) -> Result<PathBuf, StoreError> {
        validate_workflow_id(workflow_id)?;
        require_real_dir(&self.root, || StoreError::NotFound(workflow_id.to_owned()))?;
        let dir = self.root.join(workflow_id);
        require_real_dir(&dir, || StoreError::NotFound(workflow_id.to_owned()))?;
        Ok(dir)
    }

    /// Writes `bytes` to a fresh owner-only temp file in `dir` and renames
    /// it onto `target` (in the same directory, so the rename is atomic).
    fn write_via_temp(&self, dir: &Path, target: &Path, bytes: &[u8]) -> Result<(), StoreError> {
        let counter = self.temp_counter.fetch_add(1, Ordering::Relaxed);
        let temp = dir.join(format!(".tmp-{}-{counter}", std::process::id()));
        let result = (|| {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(FILE_MODE)
                .open(&temp)
                .map_err(io_error("create temp file"))?;
            file.write_all(bytes).map_err(io_error("write temp file"))?;
            file.flush().map_err(io_error("flush temp file"))?;
            fs::rename(&temp, target).map_err(io_error("rename temp file into place"))
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temp);
        }
        result
    }

    fn load_manifest(&self, workflow_id: &str, dir: &Path) -> Result<Manifest, StoreError> {
        let raw = read_no_follow(&dir.join(MANIFEST_FILE))?;

        #[derive(Deserialize)]
        struct VersionProbe {
            schema_version: u64,
        }
        let probe: VersionProbe =
            serde_json::from_slice(&raw).map_err(|error| StoreError::CorruptManifest {
                workflow_id: workflow_id.to_owned(),
                detail: error.to_string(),
            })?;
        if probe.schema_version != u64::from(SCHEMA_VERSION) {
            return Err(StoreError::UnsupportedSchemaVersion(probe.schema_version));
        }

        serde_json::from_slice(&raw).map_err(|error| StoreError::CorruptManifest {
            workflow_id: workflow_id.to_owned(),
            detail: error.to_string(),
        })
    }

    fn load_events(&self, workflow_id: &str, dir: &Path) -> Result<Vec<Event>, StoreError> {
        // `create` always writes the event log, so a missing or unreadable
        // `events.jsonl` is damage, not an empty recording; the error must
        // surface rather than silently dangling the manifest's event_ids.
        let raw = read_no_follow(&dir.join(EVENTS_FILE))?;
        let text = String::from_utf8_lossy(&raw);

        let mut lines: Vec<&str> = text.split('\n').collect();
        let torn_candidate = match lines.last() {
            // The file ends with a newline: every line is complete.
            Some(&"") => {
                lines.pop();
                None
            }
            // No trailing newline: the final line may be a torn append.
            Some(_) => lines.pop(),
            None => None,
        };

        let mut events = Vec::with_capacity(lines.len() + 1);
        for (index, line) in lines.iter().enumerate() {
            let event: Event =
                serde_json::from_str(line).map_err(|error| StoreError::CorruptEvents {
                    workflow_id: workflow_id.to_owned(),
                    line: index + 1,
                    detail: error.to_string(),
                })?;
            events.push(event);
        }
        if let Some(candidate) = torn_candidate {
            // A torn final line is skipped; a parseable one is kept.
            if let Ok(event) = serde_json::from_str::<Event>(candidate) {
                events.push(event);
            }
        }
        Ok(events)
    }
}

impl WorkflowStore for JsonWorkflowStore {
    fn create(&self, name: &str) -> Result<CreatedWorkflow, StoreError> {
        DirBuilder::new()
            .recursive(true)
            .mode(DIR_MODE)
            .create(&self.root)
            .map_err(io_error("create store root"))?;
        require_real_dir(&self.root, || {
            StoreError::Io {
                context: "store root vanished after creation".to_owned(),
                source: std::io::Error::from(std::io::ErrorKind::NotFound),
            }
        })?;

        let now = self.clock.now();
        let prefix = folder_prefix(now);

        // The mkdir is the atomic claim on the id; a suffix collision
        // retries with fresh entropy.
        let mut claimed: Option<(String, PathBuf)> = None;
        for _ in 0..CREATE_ATTEMPTS {
            let id = format!("{prefix}-{}", entropy_suffix());
            let dir = self.root.join(&id);
            match DirBuilder::new().mode(DIR_MODE).create(&dir) {
                Ok(()) => {
                    claimed = Some((id, dir));
                    break;
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(io_error("create workflow folder")(error)),
            }
        }
        let Some((id, dir)) = claimed else {
            return Err(StoreError::Io {
                context: "could not claim a unique workflow folder".to_owned(),
                source: std::io::Error::from(std::io::ErrorKind::AlreadyExists),
            });
        };
        let guard = UnpublishedWorkflow {
            id: id.clone(),
            path: dir.clone(),
            armed: true,
        };

        DirBuilder::new()
            .mode(DIR_MODE)
            .create(dir.join(SHOTS_DIR))
            .map_err(io_error("create shots folder"))?;
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(FILE_MODE)
            .open(dir.join(EVENTS_FILE))
            .map_err(io_error("create event log"))?;

        let manifest = Manifest {
            schema_version: SCHEMA_VERSION,
            id,
            name: name.to_owned(),
            created_at: manifest_timestamp(now),
            steps: Vec::new(),
        };
        let bytes = manifest_bytes(&manifest)?;
        self.write_via_temp(&dir, &dir.join(MANIFEST_FILE), &bytes)?;

        Ok(CreatedWorkflow { manifest, guard })
    }

    fn append_event(
        &self,
        workflow_id: &str,
        event: &Event,
        shots: &ShotPayloads,
    ) -> Result<(), StoreError> {
        let dir = self.workflow_dir(workflow_id)?;
        validate_event_id(&event.id)?;
        if event.shots != ShotPaths::for_event(&event.id) {
            return Err(StoreError::InvalidShotPaths {
                event_id: event.id.clone(),
            });
        }
        let shots_dir = dir.join(SHOTS_DIR);
        require_real_dir(&shots_dir, || StoreError::NotFound(workflow_id.to_owned()))?;

        // Screenshots first: a failed shot write must leave no JSONL line.
        for (relative, bytes) in [
            (&event.shots.full, &shots.full),
            (&event.shots.window, &shots.window),
            (&event.shots.element, &shots.element),
        ] {
            let file_name = Path::new(relative)
                .file_name()
                .ok_or_else(|| StoreError::InvalidShotPaths {
                    event_id: event.id.clone(),
                })?;
            self.write_via_temp(&shots_dir, &shots_dir.join(file_name), bytes)?;
        }

        let mut line = serde_json::to_string(event).map_err(|error| StoreError::Io {
            context: "serialize event line".to_owned(),
            source: std::io::Error::new(std::io::ErrorKind::InvalidData, error),
        })?;
        line.push('\n');
        let mut file = OpenOptions::new()
            .append(true)
            .custom_flags(libc::O_NOFOLLOW)
            .open(dir.join(EVENTS_FILE))
            .map_err(io_error("open event log for append"))?;
        file.write_all(line.as_bytes())
            .map_err(io_error("append event line"))?;
        file.flush().map_err(io_error("flush event log"))?;
        Ok(())
    }

    fn load(&self, workflow_id: &str) -> Result<LoadedWorkflow, StoreError> {
        let dir = self.workflow_dir(workflow_id)?;
        let manifest = self.load_manifest(workflow_id, &dir)?;
        let events = self.load_events(workflow_id, &dir)?;
        Ok(LoadedWorkflow { manifest, events })
    }

    fn save_manifest(&self, workflow_id: &str, manifest: &Manifest) -> Result<(), StoreError> {
        let dir = self.workflow_dir(workflow_id)?;
        if manifest.id != workflow_id {
            return Err(StoreError::ManifestIdMismatch {
                workflow_id: workflow_id.to_owned(),
                manifest_id: manifest.id.clone(),
            });
        }
        if manifest.schema_version != SCHEMA_VERSION {
            return Err(StoreError::UnsupportedSchemaVersion(u64::from(
                manifest.schema_version,
            )));
        }
        let bytes = manifest_bytes(manifest)?;
        self.write_via_temp(&dir, &dir.join(MANIFEST_FILE), &bytes)
    }

    fn list(&self) -> Result<Vec<WorkflowSummary>, StoreError> {
        let entries = match fs::read_dir(&self.root) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(io_error("read store root")(error)),
        };
        let mut summaries = Vec::new();
        for entry in entries {
            let entry = entry.map_err(io_error("read store root entry"))?;
            let Some(id) = entry.file_name().to_str().map(str::to_owned) else {
                continue;
            };
            if validate_workflow_id(&id).is_err() {
                continue;
            }
            // `DirEntry::file_type` does not follow symlinks, so a
            // symlinked entry is skipped rather than traversed.
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if !file_type.is_dir() {
                continue;
            }
            let Ok(manifest) = self.load_manifest(&id, &entry.path()) else {
                continue;
            };
            // A damaged or missing event log keeps the row listable with
            // placeholder duration and thumbnail (DEC-006).
            let (duration_ms, thumbnail_event_id) =
                match self.load_events(&id, &entry.path()) {
                    Ok(events) => (
                        event_span_ms(&events),
                        manifest
                            .steps
                            .first()
                            .and_then(|step| step.event_ids.first().cloned()),
                    ),
                    Err(_) => (None, None),
                };
            summaries.push(WorkflowSummary {
                id,
                name: manifest.name,
                created_at: manifest.created_at,
                step_count: manifest.steps.len() as u64,
                duration_ms,
                thumbnail_event_id,
            });
        }
        summaries.sort_by(|a, b| b.id.cmp(&a.id));
        Ok(summaries)
    }

    fn locate(&self, workflow_id: &str) -> Result<PathBuf, StoreError> {
        self.workflow_dir(workflow_id)
    }

    fn read_shot(
        &self,
        workflow_id: &str,
        event_id: &str,
        variant: ShotVariant,
    ) -> Result<Vec<u8>, StoreError> {
        let dir = self.workflow_dir(workflow_id)?;
        validate_event_id(event_id)?;
        let shots_dir = dir.join(SHOTS_DIR);
        require_real_dir(&shots_dir, || StoreError::NotFound(workflow_id.to_owned()))?;
        read_no_follow(&shots_dir.join(format!("{event_id}.{}.png", variant.file_suffix())))
    }
}

/// Milliseconds from the first to the last event timestamp; zero with
/// fewer than two events; `None` when an endpoint timestamp does not
/// parse (a damaged log line).
fn event_span_ms(events: &[Event]) -> Option<u64> {
    let (Some(first), Some(last)) = (events.first(), events.last()) else {
        return Some(0);
    };
    if events.len() < 2 {
        return Some(0);
    }
    let parse = |ts: &str| chrono::DateTime::parse_from_rfc3339(ts).ok();
    let span = parse(&last.ts)? - parse(&first.ts)?;
    Some(span.num_milliseconds().max(0) as u64)
}

fn manifest_bytes(manifest: &Manifest) -> Result<Vec<u8>, StoreError> {
    let mut bytes = serde_json::to_vec_pretty(manifest).map_err(|error| StoreError::Io {
        context: "serialize manifest".to_owned(),
        source: std::io::Error::new(std::io::ErrorKind::InvalidData, error),
    })?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn io_error(context: &str) -> impl FnOnce(std::io::Error) -> StoreError + '_ {
    move |source| StoreError::Io {
        context: context.to_owned(),
        source,
    }
}

/// Rejects ids that could escape the store root: only `[A-Za-z0-9_-]`,
/// non-empty, bounded length.
fn validate_workflow_id(id: &str) -> Result<(), StoreError> {
    let valid = !id.is_empty()
        && id.len() <= 128
        && id
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_');
    if valid {
        Ok(())
    } else {
        Err(StoreError::InvalidWorkflowId(id.to_owned()))
    }
}

fn validate_event_id(id: &str) -> Result<(), StoreError> {
    let valid = !id.is_empty()
        && id.len() <= 128
        && id
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_');
    if valid {
        Ok(())
    } else {
        Err(StoreError::InvalidEventId(id.to_owned()))
    }
}

/// Requires `path` to be an existing real directory; symlinks are
/// rejected so operations stay confined to the store root.
fn require_real_dir(
    path: &Path,
    missing: impl FnOnce() -> StoreError,
) -> Result<(), StoreError> {
    let metadata = fs::symlink_metadata(path).map_err(|_| missing())?;
    if metadata.file_type().is_symlink() {
        return Err(StoreError::SymlinkRejected(path.to_owned()));
    }
    if !metadata.is_dir() {
        return Err(StoreError::Io {
            context: format!("{} is not a directory", path.display()),
            source: std::io::Error::from(std::io::ErrorKind::NotADirectory),
        });
    }
    Ok(())
}

/// Reads a regular file without following a symlink at the final
/// component.
fn read_no_follow(path: &Path) -> Result<Vec<u8>, StoreError> {
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
        .map_err(|source| {
            if source.raw_os_error() == Some(libc::ELOOP) {
                StoreError::SymlinkRejected(path.to_owned())
            } else {
                StoreError::Io {
                    context: format!("open {}", path.display()),
                    source,
                }
            }
        })?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(io_error("read file"))?;
    Ok(bytes)
}

/// A 16-bit entropy suffix for readable folder names. Reads ambient
/// process entropy through `RandomState`; uniqueness is ultimately
/// enforced by the mkdir claim in `create`, not by this value.
fn entropy_suffix() -> String {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    let mut hasher = RandomState::new().build_hasher();
    hasher.write_u64(0x77_6f_72_6b_66_6c_6f_77);
    format!("{:04x}", hasher.finish() & 0xffff)
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::{symlink, MetadataExt};

    use chrono::TimeZone;

    use crate::domain::schema::{Classification, Step};
    use crate::recording::testutil::{
        fixed_clock, sample_click_event, sample_key_event, sample_shots,
    };

    use super::*;

    fn store_in(dir: &Path) -> JsonWorkflowStore {
        let clock = fixed_clock(
            chrono::Utc
                .with_ymd_and_hms(2026, 8, 16, 22, 31, 5)
                .unwrap(),
        );
        JsonWorkflowStore::new(dir.to_path_buf(), clock)
    }

    fn created(store: &JsonWorkflowStore, name: &str) -> (String, Manifest) {
        let created = store.create(name).unwrap();
        let manifest = created.manifest.clone();
        (created.guard.publish(), manifest)
    }

    #[test]
    fn create_lays_out_the_folder_with_an_empty_valid_manifest() {
        let temp = tempfile::tempdir().unwrap();
        let store = store_in(temp.path());
        let (id, manifest) = created(&store, "Approve invoice");

        assert!(id.starts_with("2026-08-16-223105-"), "id was {id}");
        let dir = temp.path().join(&id);
        assert!(dir.join(EVENTS_FILE).is_file());
        assert!(dir.join(MANIFEST_FILE).is_file());
        assert!(dir.join(SHOTS_DIR).is_dir());
        assert_eq!(fs::read(dir.join(EVENTS_FILE)).unwrap(), b"");

        assert_eq!(manifest.schema_version, 1);
        assert_eq!(manifest.id, id);
        assert_eq!(manifest.name, "Approve invoice");
        assert_eq!(manifest.created_at, "2026-08-16T22:31:05Z");
        assert_eq!(manifest.steps, vec![]);

        // The persisted manifest matches the returned one and loads.
        let loaded = store.load(&id).unwrap();
        assert_eq!(loaded.manifest, manifest);
        assert_eq!(loaded.events, vec![]);
    }

    #[test]
    fn created_folders_and_files_are_owner_only() {
        let temp = tempfile::tempdir().unwrap();
        // The store creates this root itself, so its mode is asserted too.
        let root = temp.path().join("workflows");
        let store = store_in(&root);
        let (id, _) = created(&store, "modes");
        let event = sample_click_event("evt_0001");
        store.append_event(&id, &event, &sample_shots()).unwrap();

        let dir = root.join(&id);
        let mode = |path: &Path| fs::metadata(path).unwrap().mode() & 0o777;
        assert_eq!(mode(&root), 0o700, "root must be owner-only");
        assert_eq!(mode(&dir), 0o700);
        assert_eq!(mode(&dir.join(SHOTS_DIR)), 0o700);
        assert_eq!(mode(&dir.join(EVENTS_FILE)), 0o600);
        assert_eq!(mode(&dir.join(MANIFEST_FILE)), 0o600);
        assert_eq!(mode(&dir.join(&event.shots.full)), 0o600);
    }

    #[test]
    fn append_event_writes_the_triple_at_the_recorded_paths_then_one_line() {
        let temp = tempfile::tempdir().unwrap();
        let store = store_in(temp.path());
        let (id, _) = created(&store, "w");
        let event = sample_click_event("evt_0001");
        let shots = sample_shots();
        store.append_event(&id, &event, &shots).unwrap();

        let dir = temp.path().join(&id);
        assert_eq!(fs::read(dir.join(&event.shots.full)).unwrap(), shots.full);
        assert_eq!(
            fs::read(dir.join(&event.shots.window)).unwrap(),
            shots.window,
        );
        assert_eq!(
            fs::read(dir.join(&event.shots.element)).unwrap(),
            shots.element,
        );

        let log = fs::read_to_string(dir.join(EVENTS_FILE)).unwrap();
        let lines: Vec<&str> = log.lines().collect();
        assert_eq!(lines.len(), 1);
        assert_eq!(serde_json::from_str::<Event>(lines[0]).unwrap(), event);
        assert!(log.ends_with('\n'));
    }

    #[test]
    fn later_appends_leave_existing_lines_byte_identical() {
        let temp = tempfile::tempdir().unwrap();
        let store = store_in(temp.path());
        let (id, _) = created(&store, "w");
        let events_path = temp.path().join(&id).join(EVENTS_FILE);

        store
            .append_event(&id, &sample_click_event("evt_0001"), &sample_shots())
            .unwrap();
        let after_first = fs::read(&events_path).unwrap();

        store
            .append_event(&id, &sample_key_event("evt_0002"), &sample_shots())
            .unwrap();
        let after_second = fs::read(&events_path).unwrap();

        assert!(after_second.starts_with(&after_first));
        assert_eq!(
            after_second.iter().filter(|&&b| b == b'\n').count(),
            2,
            "append-only log gains exactly one line",
        );
    }

    #[test]
    fn manifest_saves_replace_atomically_and_leave_no_temp_files() {
        let temp = tempfile::tempdir().unwrap();
        let store = store_in(temp.path());
        let (id, mut manifest) = created(&store, "w");

        manifest.steps.push(Step {
            id: "step_0001".into(),
            event_ids: vec!["evt_0001".into()],
            classification: Classification::Click,
            title: "Click \"OK\" — TextEdit".into(),
            description: String::new(),
        });
        store.save_manifest(&id, &manifest).unwrap();
        manifest.name = "renamed".into();
        store.save_manifest(&id, &manifest).unwrap();

        let dir = temp.path().join(&id);
        let entries: Vec<String> = fs::read_dir(&dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        let mut sorted = entries.clone();
        sorted.sort();
        assert_eq!(sorted, vec![EVENTS_FILE, SHOTS_DIR, MANIFEST_FILE]);

        let loaded = store.load(&id).unwrap();
        assert_eq!(loaded.manifest, manifest);
        assert_eq!(loaded.manifest.steps[0].event_ids, vec!["evt_0001"]);
    }

    #[test]
    fn save_manifest_rejects_a_manifest_for_a_different_workflow() {
        let temp = tempfile::tempdir().unwrap();
        let store = store_in(temp.path());
        let (id_a, manifest_a) = created(&store, "a");
        let (id_b, manifest_b) = created(&store, "b");

        let error = store.save_manifest(&id_a, &manifest_b).unwrap_err();
        assert!(
            matches!(&error, StoreError::ManifestIdMismatch { workflow_id, manifest_id }
                if *workflow_id == id_a && *manifest_id == id_b),
            "got {error}",
        );
        // Both manifests are untouched.
        assert_eq!(store.load(&id_a).unwrap().manifest, manifest_a);
        assert_eq!(store.load(&id_b).unwrap().manifest, manifest_b);
    }

    #[test]
    fn save_manifest_rejects_a_non_v1_schema_version() {
        let temp = tempfile::tempdir().unwrap();
        let store = store_in(temp.path());
        let (id, mut manifest) = created(&store, "w");
        manifest.schema_version = 2;

        let error = store.save_manifest(&id, &manifest).unwrap_err();
        assert!(
            matches!(error, StoreError::UnsupportedSchemaVersion(2)),
            "got {error}",
        );
        // The stored manifest keeps version 1 and stays loadable.
        assert_eq!(store.load(&id).unwrap().manifest.schema_version, 1);
    }

    #[test]
    fn a_missing_event_log_fails_load_instead_of_reporting_empty() {
        let temp = tempfile::tempdir().unwrap();
        let store = store_in(temp.path());
        let (id, _) = created(&store, "w");
        store
            .append_event(&id, &sample_click_event("evt_0001"), &sample_shots())
            .unwrap();
        fs::remove_file(temp.path().join(&id).join(EVENTS_FILE)).unwrap();

        let error = store.load(&id).unwrap_err();
        assert!(matches!(error, StoreError::Io { .. }), "got {error}");
        // The listing still shows the workflow: its name and creation time
        // come from the intact manifest.
        let list = store.list().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, id);
    }

    #[test]
    fn load_rejects_an_unsupported_schema_version_explicitly() {
        let temp = tempfile::tempdir().unwrap();
        let store = store_in(temp.path());
        let (id, _) = created(&store, "w");
        let manifest_path = temp.path().join(&id).join(MANIFEST_FILE);
        fs::write(
            &manifest_path,
            br#"{"schema_version":2,"id":"x","name":"x","created_at":"2026-08-16T22:31:05Z","steps":[]}"#,
        )
        .unwrap();

        let error = store.load(&id).unwrap_err();
        assert!(
            matches!(error, StoreError::UnsupportedSchemaVersion(2)),
            "got {error}",
        );
    }

    #[test]
    fn load_skips_a_torn_final_jsonl_line() {
        let temp = tempfile::tempdir().unwrap();
        let store = store_in(temp.path());
        let (id, _) = created(&store, "w");
        store
            .append_event(&id, &sample_click_event("evt_0001"), &sample_shots())
            .unwrap();
        store
            .append_event(&id, &sample_key_event("evt_0002"), &sample_shots())
            .unwrap();

        let events_path = temp.path().join(&id).join(EVENTS_FILE);
        let mut file = OpenOptions::new().append(true).open(&events_path).unwrap();
        file.write_all(br#"{"id":"evt_0003","ts":"2026-"#).unwrap();
        drop(file);

        let loaded = store.load(&id).unwrap();
        let ids: Vec<&str> = loaded.events.iter().map(|e| e.id.as_str()).collect();
        assert_eq!(ids, vec!["evt_0001", "evt_0002"]);
    }

    #[test]
    fn orphan_shots_break_neither_load_nor_list() {
        let temp = tempfile::tempdir().unwrap();
        let store = store_in(temp.path());
        let (id, _) = created(&store, "w");
        store
            .append_event(&id, &sample_click_event("evt_0001"), &sample_shots())
            .unwrap();
        fs::write(
            temp.path().join(&id).join(SHOTS_DIR).join("orphan.png"),
            b"stray",
        )
        .unwrap();

        assert_eq!(store.load(&id).unwrap().events.len(), 1);
        let list = store.list().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, id);
    }

    #[test]
    fn shot_write_failure_leaves_no_jsonl_line() {
        let temp = tempfile::tempdir().unwrap();
        let store = store_in(temp.path());
        let (id, _) = created(&store, "w");
        store
            .append_event(&id, &sample_click_event("evt_0001"), &sample_shots())
            .unwrap();
        let events_path = temp.path().join(&id).join(EVENTS_FILE);
        let before = fs::read(&events_path).unwrap();

        // Replace shots/ with a symlink: the confinement check fails the
        // shot write before any JSONL append.
        let shots_dir = temp.path().join(&id).join(SHOTS_DIR);
        let real_target = temp.path().join("elsewhere");
        fs::create_dir(&real_target).unwrap();
        fs::remove_dir_all(&shots_dir).unwrap();
        symlink(&real_target, &shots_dir).unwrap();

        let error = store
            .append_event(&id, &sample_key_event("evt_0002"), &sample_shots())
            .unwrap_err();
        assert!(matches!(error, StoreError::SymlinkRejected(_)), "got {error}");
        assert_eq!(fs::read(&events_path).unwrap(), before);
    }

    #[test]
    fn traversal_workflow_ids_are_rejected() {
        let temp = tempfile::tempdir().unwrap();
        let store = store_in(temp.path());
        for id in ["", "..", "../x", "a/b", "a\\b", ".hidden", "a b"] {
            let error = store.load(id).unwrap_err();
            assert!(
                matches!(error, StoreError::InvalidWorkflowId(_)),
                "id {id:?} got {error}",
            );
        }
    }

    #[test]
    fn symlinked_workflow_dir_is_rejected() {
        let temp = tempfile::tempdir().unwrap();
        let store = store_in(temp.path());
        let real_target = temp.path().join("real");
        fs::create_dir_all(real_target.join(SHOTS_DIR)).unwrap();
        symlink(&real_target, temp.path().join("linked")).unwrap();

        let error = store.load("linked").unwrap_err();
        assert!(matches!(error, StoreError::SymlinkRejected(_)), "got {error}");
        let error = store
            .append_event("linked", &sample_click_event("evt_0001"), &sample_shots())
            .unwrap_err();
        assert!(matches!(error, StoreError::SymlinkRejected(_)), "got {error}");
    }

    #[test]
    fn non_canonical_shot_paths_are_rejected_before_any_write() {
        let temp = tempfile::tempdir().unwrap();
        let store = store_in(temp.path());
        let (id, _) = created(&store, "w");
        let mut event = sample_click_event("evt_0001");
        event.shots.full = "shots/../escape.png".into();

        let error = store
            .append_event(&id, &event, &sample_shots())
            .unwrap_err();
        assert!(
            matches!(error, StoreError::InvalidShotPaths { .. }),
            "got {error}",
        );
        assert_eq!(
            fs::read_dir(temp.path().join(&id).join(SHOTS_DIR))
                .unwrap()
                .count(),
            0,
        );
        assert_eq!(fs::read(temp.path().join(&id).join(EVENTS_FILE)).unwrap(), b"");
    }

    #[test]
    fn dropping_the_unpublished_guard_rolls_the_folder_back() {
        let temp = tempfile::tempdir().unwrap();
        let store = store_in(temp.path());

        let created = store.create("doomed").unwrap();
        let dir = temp.path().join(created.guard.id());
        assert!(dir.is_dir());
        drop(created);
        assert!(!dir.exists());

        let kept = store.create("kept").unwrap();
        let id = kept.guard.publish();
        assert!(temp.path().join(&id).is_dir());
    }

    #[test]
    fn list_returns_summaries_newest_first() {
        let temp = tempfile::tempdir().unwrap();
        let store = store_in(temp.path());
        let (id_a, _) = created(&store, "first");
        let (id_b, _) = created(&store, "second");

        let list = store.list().unwrap();
        let ids: Vec<&str> = list.iter().map(|s| s.id.as_str()).collect();
        let mut expected = vec![id_a.as_str(), id_b.as_str()];
        expected.sort();
        expected.reverse();
        assert_eq!(ids, expected, "ids sort descending: newest first");
        assert_eq!(list.len(), 2);
        for summary in &list {
            assert_eq!(summary.created_at, "2026-08-16T22:31:05Z");
        }
    }

    /// Saves a manifest whose steps reference the given events 1:1.
    fn save_steps(store: &JsonWorkflowStore, id: &str, mut manifest: Manifest, events: &[&Event]) {
        manifest.steps = events
            .iter()
            .enumerate()
            .map(|(index, event)| Step {
                id: format!("step_{:04}", index + 1),
                event_ids: vec![event.id.clone()],
                classification: Classification::Click,
                title: format!("Step {}", index + 1),
                description: String::new(),
            })
            .collect();
        store.save_manifest(id, &manifest).unwrap();
    }

    #[test]
    fn summaries_carry_step_count_duration_and_thumbnail_reference() {
        let temp = tempfile::tempdir().unwrap();
        let store = store_in(temp.path());
        let (id, manifest) = created(&store, "w");

        let mut first = sample_click_event("evt_0001");
        first.ts = "2026-08-16T22:31:05.100Z".to_owned();
        let mut second = sample_key_event("evt_0002");
        second.ts = "2026-08-16T22:31:06.000Z".to_owned();
        let mut third = sample_click_event("evt_0003");
        third.ts = "2026-08-16T22:31:23.350Z".to_owned();
        for event in [&first, &second, &third] {
            store.append_event(&id, event, &sample_shots()).unwrap();
        }
        save_steps(&store, &id, manifest, &[&first, &second, &third]);

        let list = store.list().unwrap();
        assert_eq!(list.len(), 1);
        let summary = &list[0];
        assert_eq!(summary.step_count, 3, "step count comes from manifest steps");
        assert_eq!(summary.duration_ms, Some(18_250), "first-to-last event span");
        assert_eq!(
            summary.thumbnail_event_id.as_deref(),
            Some("evt_0001"),
            "thumbnail references the first step's first event",
        );
    }

    #[test]
    fn zero_and_single_event_workflows_report_zero_duration() {
        let temp = tempfile::tempdir().unwrap();
        let store = store_in(temp.path());
        let (empty_id, _) = created(&store, "empty");
        let (single_id, single_manifest) = created(&store, "single");
        let event = sample_click_event("evt_0001");
        store
            .append_event(&single_id, &event, &sample_shots())
            .unwrap();
        save_steps(&store, &single_id, single_manifest, &[&event]);

        let list = store.list().unwrap();
        let by_id = |id: &str| {
            list.iter()
                .find(|summary| summary.id == id)
                .expect("summary listed")
        };
        let empty = by_id(&empty_id);
        assert_eq!(empty.step_count, 0);
        assert_eq!(empty.duration_ms, Some(0));
        assert_eq!(empty.thumbnail_event_id, None, "no step, no thumbnail");
        let single = by_id(&single_id);
        assert_eq!(single.step_count, 1);
        assert_eq!(single.duration_ms, Some(0));
        assert_eq!(single.thumbnail_event_id.as_deref(), Some("evt_0001"));
    }

    #[test]
    fn a_damaged_event_log_keeps_the_row_listed_with_placeholders() {
        let temp = tempfile::tempdir().unwrap();
        let store = store_in(temp.path());
        let (id, manifest) = created(&store, "damaged");
        let event = sample_click_event("evt_0001");
        store.append_event(&id, &event, &sample_shots()).unwrap();
        save_steps(&store, &id, manifest, &[&event]);
        // A corrupt newline-terminated line makes the log unreadable.
        fs::write(
            temp.path().join(&id).join(EVENTS_FILE),
            b"{not json}\n",
        )
        .unwrap();

        let list = store.list().unwrap();
        assert_eq!(list.len(), 1);
        let summary = &list[0];
        assert_eq!(summary.step_count, 1, "step count still comes from the manifest");
        assert_eq!(summary.duration_ms, None, "unreadable log omits the duration");
        assert_eq!(summary.thumbnail_event_id, None, "unreadable log omits the thumbnail");

        // A missing log degrades the same way.
        fs::remove_file(temp.path().join(&id).join(EVENTS_FILE)).unwrap();
        let list = store.list().unwrap();
        assert_eq!(list[0].duration_ms, None);
        assert_eq!(list[0].thumbnail_event_id, None);
    }

    #[test]
    fn read_shot_returns_the_canonical_bytes_per_variant() {
        let temp = tempfile::tempdir().unwrap();
        let store = store_in(temp.path());
        let (id, _) = created(&store, "w");
        let event = sample_click_event("evt_0001");
        let shots = sample_shots();
        store.append_event(&id, &event, &shots).unwrap();

        for (variant, expected) in [
            (ShotVariant::Full, &shots.full),
            (ShotVariant::Window, &shots.window),
            (ShotVariant::Element, &shots.element),
        ] {
            assert_eq!(&store.read_shot(&id, "evt_0001", variant).unwrap(), expected);
        }
    }

    #[test]
    fn read_shot_rejects_non_canonical_ids_and_missing_targets() {
        let temp = tempfile::tempdir().unwrap();
        let store = store_in(temp.path());
        let (id, _) = created(&store, "w");
        store
            .append_event(&id, &sample_click_event("evt_0001"), &sample_shots())
            .unwrap();

        for bad_event_id in ["", "..", "../evt_0001", "evt/0001", "evt_0001.full"] {
            let error = store
                .read_shot(&id, bad_event_id, ShotVariant::Full)
                .unwrap_err();
            assert!(
                matches!(error, StoreError::InvalidEventId(_)),
                "event id {bad_event_id:?} got {error}",
            );
        }
        let error = store
            .read_shot("missing", "evt_0001", ShotVariant::Full)
            .unwrap_err();
        assert!(matches!(error, StoreError::NotFound(_)), "got {error}");
        let error = store
            .read_shot(&id, "evt_9999", ShotVariant::Full)
            .unwrap_err();
        assert!(matches!(error, StoreError::Io { .. }), "got {error}");
    }

    #[test]
    fn read_shot_refuses_symlinked_paths() {
        let temp = tempfile::tempdir().unwrap();
        let store = store_in(temp.path());
        let (id, _) = created(&store, "w");
        store
            .append_event(&id, &sample_click_event("evt_0001"), &sample_shots())
            .unwrap();

        // A symlinked PNG at the canonical path is refused (O_NOFOLLOW).
        let shots_dir = temp.path().join(&id).join(SHOTS_DIR);
        let linked = shots_dir.join("evt_0002.full.png");
        symlink(shots_dir.join("evt_0001.full.png"), &linked).unwrap();
        let error = store
            .read_shot(&id, "evt_0002", ShotVariant::Full)
            .unwrap_err();
        assert!(matches!(error, StoreError::SymlinkRejected(_)), "got {error}");

        // A symlinked shots directory is refused before any open.
        let real_target = temp.path().join("elsewhere");
        fs::create_dir(&real_target).unwrap();
        fs::remove_dir_all(&shots_dir).unwrap();
        symlink(&real_target, &shots_dir).unwrap();
        let error = store
            .read_shot(&id, "evt_0001", ShotVariant::Full)
            .unwrap_err();
        assert!(matches!(error, StoreError::SymlinkRejected(_)), "got {error}");
    }

    #[test]
    fn shot_variants_parse_from_the_allowlist_only() {
        assert_eq!("full".parse::<ShotVariant>().unwrap(), ShotVariant::Full);
        assert_eq!("window".parse::<ShotVariant>().unwrap(), ShotVariant::Window);
        assert_eq!(
            "element".parse::<ShotVariant>().unwrap(),
            ShotVariant::Element,
        );
        for bad in ["", "Full", "screen", "window.png", "../window"] {
            assert!(
                bad.parse::<ShotVariant>().is_err(),
                "variant {bad:?} must be rejected",
            );
        }
    }

    #[test]
    fn locate_validates_before_returning_the_folder_path() {
        let temp = tempfile::tempdir().unwrap();
        let store = store_in(temp.path());
        let (id, _) = created(&store, "w");

        assert_eq!(store.locate(&id).unwrap(), temp.path().join(&id));
        for bad in ["", "..", "../x", "a/b"] {
            let error = store.locate(bad).unwrap_err();
            assert!(
                matches!(error, StoreError::InvalidWorkflowId(_)),
                "id {bad:?} got {error}",
            );
        }
        let error = store.locate("missing").unwrap_err();
        assert!(matches!(error, StoreError::NotFound(_)), "got {error}");

        let real_target = temp.path().join("real");
        fs::create_dir(&real_target).unwrap();
        symlink(&real_target, temp.path().join("linked")).unwrap();
        let error = store.locate("linked").unwrap_err();
        assert!(matches!(error, StoreError::SymlinkRejected(_)), "got {error}");
    }
}
