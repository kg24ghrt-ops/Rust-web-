Project Development Instructions

Build and Compilation Policy

All project compilation, building, packaging, and release verification must happen exclusively through GitHub Actions CI/CD.

Never perform local builds or compilations.

1. CI/CD-Only Builds

- Use GitHub Actions as the only build and compilation environment.
- Never compile, package, or build this project locally.
- Do not install large SDKs, compilers, toolchains, or build dependencies on the local device.
- Do not attempt to bypass CI/CD by running alternative local build commands.
- Use the existing GitHub Actions workflows whenever possible.
- If a workflow is missing or outdated, inspect the project and implement or update the appropriate workflow.

2. GitHub Integration

Use GitHub to manage and inspect the project's development lifecycle.

- Inspect repository structure, branches, commits, and workflow configurations.
- Monitor GitHub Actions execution and build logs.
- Inspect CI failures and identify their root causes.
- Verify successful builds using GitHub Actions results and generated artifacts.
- Use GitHub-hosted runners for all resource-intensive build operations.
- Prefer official GitHub documentation and current action versions.

3. Development Workflow

For every significant change:

1. Inspect the relevant source files and existing architecture.
2. Understand the existing implementation before editing.
3. Make focused changes without unnecessarily rewriting working components.
4. Run lightweight static checks and syntax validation locally when practical. These must not compile or build the project.
5. Commit and push changes when authorized.
6. Trigger or monitor the appropriate GitHub Actions workflow.
7. Inspect the resulting logs and artifacts.
8. Report the actual CI/CD result.

4. Git and Repository Issues

I will assist with Git-related problems when necessary.

If you encounter:

- Authentication failures.
- SSH or HTTPS connectivity problems.
- Merge conflicts.
- Branch protection restrictions.
- Push or pull failures.
- Repository permission issues.

Stop before attempting destructive workarounds.

Explain the exact problem, provide the relevant error output, and tell me what action is needed. I will handle issues that require my intervention.

Never discard uncommitted work or force-push without explicit authorization.

5. Resource Constraints

The development environment is a phone running Alpine Linux.

Prioritize:

- Low memory consumption.
- Minimal local dependencies.
- Lightweight development tools.
- Remote CI/CD execution.
- Efficient Git operations.

Do not assume a desktop development environment is available.

6. Final Reporting

For every completed task, report:

- Files changed.
- Technical changes made.
- Git commit or push status.
- GitHub Actions workflow status.
- Build and test results.
- Artifact locations, when applicable.
- Remaining issues or required user intervention.

Core principle: Develop locally, compile remotely, verify through GitHub Actions.