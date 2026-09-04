/**
 * Design-preview seed for the desktop shell.
 *
 * This is **sample content**, not core data: it exists so the shell's layout can
 * be seen and reviewed before the core binds the capabilities behind it. It
 * lives in the host app, never in `@cronus/ui`, and nothing here is presented as
 * a real reading — the moment a real projection arrives, its `preview*` value is
 * deleted rather than merged, so there is never a moment where fabricated and
 * live numbers sit in the same object.
 */

import type { BudgetMeter, DashboardProjection, FileNode, FloorTab, SidebarTab } from "@cronus/ui";

export const previewFloors: FloorTab[] = [
  {
    id: "home",
    name: "Home",
    kind: "home",
    state: "idle",
  },
  {
    id: "core",
    name: "cronus-core",
    kind: "project",
    state: "active",
  },
  {
    id: "nodus",
    name: "nodus-runtime",
    kind: "project",
    state: "hibernating",
  },
];

/** Alert counts — unread or failing, rendered as filled pills. */
export const previewBadges: Partial<Record<SidebarTab, number>> = {
  chat: 2,
  sessions: 2,
  inbox: 3,
  kanban: 3,
  security: 1,
};

/** Tallies — how many of a thing exist, rendered as muted text. */
export const previewTallies: Partial<Record<SidebarTab, string | number>> = {
  employees: 6,
  schedule: 1,
  automation: 3,
  channels: "4/13",
};

export const previewMarkers: SidebarTab[] = [
  "wiki",
];

export const previewFileTree: FileNode[] = [
  {
    name: "crates",
    kind: "dir",
  },
  {
    name: "packages",
    kind: "dir",
  },
  {
    name: "apps",
    kind: "dir",
  },
  {
    name: "target",
    kind: "dir",
    ignored: true,
  },
  {
    name: "node_modules",
    kind: "dir",
    ignored: true,
  },
  {
    name: "Cargo.toml",
    kind: "file",
  },
  {
    name: "package.json",
    kind: "file",
  },
  {
    name: "README.md",
    kind: "file",
  },
];

export const previewRecentOffices = [
  {
    id: "core",
    name: "cronus-core",
    hint: "active",
  },
  {
    id: "nodus",
    name: "nodus-runtime",
    hint: "hibernating",
  },
];

export const previewSessionBudget: BudgetMeter = {
  percent: 69,
  label: "69% · 5h",
};
export const previewWeeklyBudget: BudgetMeter = {
  percent: 3,
  label: "3% wk",
  critical: true,
};

const bars = (
  values: [
    number,
    number,
  ][],
  labels: string[],
) =>
  values.map(([a, b], i) => ({
    label: labels[i] ?? "",
    a,
    b,
  }));

const DAYS = [
  "Mon",
  "Tue",
  "Wed",
  "Thu",
  "Fri",
  "Sat",
  "Sun",
  "Now",
];

export const previewDashboard: DashboardProjection = {
  range: "Last 7 days",
  cards: [
    {
      id: "agents",
      value: "6",
      label: "Agents",
    },
    {
      id: "sessions",
      value: "24",
      label: "Sessions",
    },
    {
      id: "messages",
      value: "1,284",
      label: "Messages",
    },
    {
      id: "tokens",
      value: "3.4M",
      label: "Tokens",
    },
    {
      id: "tools",
      value: "412",
      label: "Tool Calls",
    },
    {
      id: "cost",
      value: "$18.60",
      label: "Spend",
    },
  ],
  trends: [
    {
      id: "messages",
      title: "Message Trend",
      a: {
        label: "User Messages",
        tone: 1,
      },
      b: {
        label: "Assistant Messages",
        tone: 2,
      },
      points: bars(
        [
          [
            42,
            58,
          ],
          [
            61,
            74,
          ],
          [
            38,
            51,
          ],
          [
            77,
            92,
          ],
          [
            55,
            68,
          ],
          [
            21,
            29,
          ],
          [
            17,
            24,
          ],
          [
            64,
            81,
          ],
        ],
        DAYS,
      ),
    },
    {
      id: "sessions",
      title: "Session Trend",
      a: {
        label: "New Sessions",
        tone: 5,
      },
      b: {
        label: "Active Sessions",
        tone: 1,
      },
      points: bars(
        [
          [
            4,
            7,
          ],
          [
            6,
            9,
          ],
          [
            3,
            6,
          ],
          [
            8,
            12,
          ],
          [
            5,
            10,
          ],
          [
            2,
            4,
          ],
          [
            1,
            3,
          ],
          [
            7,
            11,
          ],
        ],
        DAYS,
      ),
    },
    {
      id: "tokens",
      title: "Token Trend",
      a: {
        label: "Prompt Tokens",
        tone: 3,
      },
      b: {
        label: "Completion Tokens",
        tone: 4,
      },
      points: bars(
        [
          [
            180,
            96,
          ],
          [
            240,
            130,
          ],
          [
            150,
            88,
          ],
          [
            310,
            165,
          ],
          [
            220,
            118,
          ],
          [
            80,
            44,
          ],
          [
            60,
            33,
          ],
          [
            265,
            142,
          ],
        ],
        DAYS,
      ),
    },
    {
      id: "calls",
      title: "LLM & Tool Call Trend",
      a: {
        label: "LLM Calls",
        tone: 1,
      },
      b: {
        label: "Tool Calls",
        tone: 2,
      },
      points: bars(
        [
          [
            31,
            48,
          ],
          [
            44,
            66,
          ],
          [
            27,
            39,
          ],
          [
            58,
            87,
          ],
          [
            39,
            58,
          ],
          [
            14,
            21,
          ],
          [
            11,
            16,
          ],
          [
            47,
            71,
          ],
        ],
        DAYS,
      ),
    },
  ],
};
