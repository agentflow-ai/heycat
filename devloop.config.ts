export default {
  tcr: {
    maxFailures: 5,
    wipPrefix: "WIP: ",
    stateFile: ".tcr-state.json",
  },

  agile: {
    customTemplates: {},
    review: {
      instructionsFile: "agile/review.md",
    },
    featureReview: {
      instructionsFile: "",
      smokeTestCommand: "",
      templateFile: "",
      minIntegrationTests: 1,
      flagMockedIntegrations: true,
    },
    linear: {
      teamId: "cb5ea72f-7c34-4cd9-9d22-9bd49074f3f2",
      stateMapping: {
        "1-backlog": "Backlog",
        "2-todo": "Todo",
        "3-in-progress": "In Progress",
        "4-review": "In Review",
        "5-done": "Done",
      },
      labels: {
        feature: "feature",
        bug: "bug",
        spec: "spec",
        fix: "fix",
        specStatuses: {
          pending: "spec:pending",
          inProgress: "spec:in-progress",
          inReview: "spec:in-review",
          completed: "spec:completed",
        },
        fixStatuses: {
          pending: "fix:pending",
          inProgress: "fix:in-progress",
          inReview: "fix:in-review",
          completed: "fix:completed",
        },
      },
    },
  },
};
