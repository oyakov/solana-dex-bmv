# Agent Workflows

This document describes the custom agent workflows available in this project to assist with development and project management.

## Available Workflows

### 1. `/linear-project-report`

Generates a comprehensive report of the current project status on Linear. It analyzes active projects, teams, and issues to provide a high-level summary.

### 2. `/linear-sync-new-work`

Synchronizes locally completed work with Linear.

- Identifies completed tasks in the codebase.
- Creates missing issues in Linear if they represent new work.
- Transitions issues to "Done" in Linear.

### 3. `/docker-testing`

Automates the build and verification of the bot within a Docker container.

- Builds the Docker image.
- Runs the container with a check flag.
- Verifies logs for successful initialization.

### 4. `/run-comprehensive-testing`

Executes a full suite of tests including unit and integration verification.

- Runs `cargo test`.
- Generates a consolidated test report.

### 5. `/update-documentation`

Ensures all documentation in the `/docs` folder and the root `README.md` is comprehensive, accurate, and synchronized with the latest codebase changes.

## How to Use

These workflows are triggered using the slash command in your interaction with the AI agent.
Example: `/linear-project-report`
