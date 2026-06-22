//! Preset role catalog — 25 built-in roles across 5 categories.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoleCategory {
    Engineering,
    Quality,
    OpsAndDocs,
    Memory,
    Business,
    Custom,
}

impl RoleCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            RoleCategory::Engineering => "engineering",
            RoleCategory::Quality => "quality",
            RoleCategory::OpsAndDocs => "ops_and_docs",
            RoleCategory::Memory => "memory",
            RoleCategory::Business => "business",
            RoleCategory::Custom => "custom",
        }
    }
}

#[derive(Debug, Clone)]
pub struct PresetRole {
    pub id: &'static str,
    pub name: &'static str,
    pub category: RoleCategory,
    pub description: &'static str,
}

pub static PRESET_CATALOG: &[PresetRole] = &[
    // Engineering (5)
    PresetRole {
        id: "architect",
        name: "Architect",
        category: RoleCategory::Engineering,
        description: "System design, architectural decisions, and technical strategy",
    },
    PresetRole {
        id: "backend-engineer",
        name: "Backend Engineer",
        category: RoleCategory::Engineering,
        description: "Server-side development, APIs, and data modeling",
    },
    PresetRole {
        id: "frontend-engineer",
        name: "Frontend Engineer",
        category: RoleCategory::Engineering,
        description: "UI development, accessibility, and browser compatibility",
    },
    PresetRole {
        id: "api-designer",
        name: "API Designer",
        category: RoleCategory::Engineering,
        description: "REST, GraphQL, and RPC API design and documentation",
    },
    PresetRole {
        id: "sql-expert",
        name: "SQL Expert",
        category: RoleCategory::Engineering,
        description: "Database schema design, query optimization, and migrations",
    },
    // Quality (7)
    PresetRole {
        id: "code-reviewer",
        name: "Code Reviewer",
        category: RoleCategory::Quality,
        description: "Code review, best practices, and feedback",
    },
    PresetRole {
        id: "test-writer",
        name: "Test Writer",
        category: RoleCategory::Quality,
        description: "Unit, integration, and end-to-end test authoring",
    },
    PresetRole {
        id: "debugger",
        name: "Debugger",
        category: RoleCategory::Quality,
        description: "Root-cause analysis and bug investigation",
    },
    PresetRole {
        id: "refactorer",
        name: "Refactorer",
        category: RoleCategory::Quality,
        description: "Code structure improvement while preserving behavior",
    },
    PresetRole {
        id: "performance-optimizer",
        name: "Performance Optimizer",
        category: RoleCategory::Quality,
        description: "Profiling, benchmarking, and performance tuning",
    },
    PresetRole {
        id: "security-auditor",
        name: "Security Auditor",
        category: RoleCategory::Quality,
        description: "Security review, vulnerability assessment, and hardening",
    },
    PresetRole {
        id: "accessibility-auditor",
        name: "Accessibility Auditor",
        category: RoleCategory::Quality,
        description: "WCAG compliance, screen reader compatibility, and inclusive design",
    },
    // Ops & Docs (5)
    PresetRole {
        id: "devops-engineer",
        name: "DevOps Engineer",
        category: RoleCategory::OpsAndDocs,
        description: "CI/CD, infrastructure, and deployment automation",
    },
    PresetRole {
        id: "incident-responder",
        name: "Incident Responder",
        category: RoleCategory::OpsAndDocs,
        description: "Incident triage, mitigation, and post-mortem",
    },
    PresetRole {
        id: "doc-writer",
        name: "Documentation Writer",
        category: RoleCategory::OpsAndDocs,
        description: "Technical writing, README, and API documentation",
    },
    PresetRole {
        id: "data-analyst",
        name: "Data Analyst",
        category: RoleCategory::OpsAndDocs,
        description: "Data querying, reporting, and visualization",
    },
    PresetRole {
        id: "prompt-engineer",
        name: "Prompt Engineer",
        category: RoleCategory::OpsAndDocs,
        description: "LLM prompt design, optimization, and evaluation",
    },
    // Memory (1)
    PresetRole {
        id: "archivist",
        name: "Archivist",
        category: RoleCategory::Memory,
        description: "Memory curation, consolidation, and trust scoring",
    },
    // Business (5)
    PresetRole {
        id: "finance",
        name: "Finance",
        category: RoleCategory::Business,
        description: "Budgeting, cost analysis, and financial reporting",
    },
    PresetRole {
        id: "hr",
        name: "HR",
        category: RoleCategory::Business,
        description: "Hiring workflows, onboarding, and people management",
    },
    PresetRole {
        id: "marketing",
        name: "Marketing",
        category: RoleCategory::Business,
        description: "Content creation, campaigns, and growth analysis",
    },
    PresetRole {
        id: "support",
        name: "Support",
        category: RoleCategory::Business,
        description: "Customer support, ticket triage, and escalation",
    },
    PresetRole {
        id: "game-dev",
        name: "Game Developer",
        category: RoleCategory::Business,
        description: "Game design, mechanics, and interactive storytelling",
    },
    // Additional engineering (2 to reach 25)
    PresetRole {
        id: "mobile-engineer",
        name: "Mobile Engineer",
        category: RoleCategory::Engineering,
        description: "iOS/Android development and cross-platform mobile apps",
    },
    PresetRole {
        id: "ml-engineer",
        name: "ML Engineer",
        category: RoleCategory::Engineering,
        description: "Machine learning model training, fine-tuning, and deployment",
    },
];
