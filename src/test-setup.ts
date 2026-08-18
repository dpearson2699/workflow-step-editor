// Vitest setup: unmount React trees between tests (globals are off, so
// Testing Library cannot register its own afterEach hook).

import { afterEach } from "vitest";
import { cleanup } from "@testing-library/react";

afterEach(cleanup);
