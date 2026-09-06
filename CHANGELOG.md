# Changelog

## [0.0.5](https://github.com/bearbearean/nbsp/compare/v0.0.4...v0.0.5) - 2026-09-06

### Added

- align the user profile design with the forms ([#25](https://github.com/bearbearean/nbsp/pull/25))
- add security policy
- add invite code page for users ([#21](https://github.com/bearbearean/nbsp/pull/21))
- add http_requests_duration_seconds prometheus metric ([#16](https://github.com/bearbearean/nbsp/pull/16))
- automatically delete refresh tokens after 30 days ([#12](https://github.com/bearbearean/nbsp/pull/12))

### Fixed

- position the footer at the bottom everywhere
- show feedback on erroneous form submits ([#17](https://github.com/bearbearean/nbsp/pull/17))
- make all database migrations use if not exists and on conflict do nothing ([#19](https://github.com/bearbearean/nbsp/pull/19))

### Other

- point dependabot to the dev branch instead of main
- add granting invites SQL snippets
- implement FromRequestParts for all auth middleware ([#24](https://github.com/bearbearean/nbsp/pull/24))
- moduralize the axum routes ([#23](https://github.com/bearbearean/nbsp/pull/23))
- put the common <header> into a macro for easier re-use ([#22](https://github.com/bearbearean/nbsp/pull/22))
- consolidate jwt and auth code into one module and clean it up ([#20](https://github.com/bearbearean/nbsp/pull/20))
- *(dev)* add a pg-isready check before makers dev
- lower jwt expiry to 15 minutes
- start on development docs

## [0.0.4](https://github.com/bearbearean/nbsp/compare/v0.0.3...v0.0.4) - 2026-08-22

### Added

- add prometheus metrics endpoint and http requests tracking
- add logging out to user's own profiles
- add a very basic user profile with their join date
- add a configurable content security policy
- add a redirect parameter for returning back to the previous url after login ([#13](https://github.com/bearbearean/nbsp/pull/13))
- add account login form
- implement user sessions using jsonwebtoken and refresh tokens
- include user-agent for logs with CustomMakeSpan
- basic user registration using an invite code

### Fixed

- set migration filename
- properly remove cookies by setting the same path
- hide the register and login boxes on the home page when logged in
- put the redirect url in the login form too, so it works correctly ([#13](https://github.com/bearbearean/nbsp/pull/13))
- redirect already logged in users away from the login and register pages ([#14](https://github.com/bearbearean/nbsp/pull/14))
- hide sensitive headers from appearing in tracing output ([#10](https://github.com/bearbearean/nbsp/pull/10))
- correct the position of the footer on the HTTP status page ([#9](https://github.com/bearbearean/nbsp/pull/9))'
- add a cachebusting string to nbsp.css with the nbsp version number ([#7](https://github.com/bearbearean/nbsp/pull/7))

### Other

- add caching rust dependencies
- remove extra debug log
- create the Database tips section
- set RUST_LOG=trace for the makers dev task
- add note about dev branch PRs
- add dependency review workflow
- add a note to not modify username/password length requirements
- lower the CustomMakeSpan to the INFO level for logging
- initial anti-goals drafting
- add utm_campaign to instances links
- output database migration files using full timestamps

## [0.0.3](https://github.com/bearbearean/nbsp/compare/v0.0.2...v0.0.3) - 2026-08-18

### Added

- add a basic footer with the nbsp version number
- *(nbsp_config)* add a way to inject extra html in the head and body
- add a generic permanent redirects handler, starting with /robots.txt -> /assets/robots.txt

### Fixed

- propagate x-request-id to responses

### Other

- add nbsp_config documentation
- pass NbspConfig to templates directly

## [0.0.2](https://github.com/bearbearean/nbsp/compare/v0.0.1...v0.0.2) - 2026-08-17

### Added

- add 2 options to nbsp_config for configuring the instance title and subtitle

### Other

- dev branch ([#4](https://github.com/bearbearean/nbsp/pull/4))
- remove unneeded testing homepage notice

## [0.0.1](https://github.com/bearbearean/nbsp/compare/v0.0.0...v0.0.1) - 2026-08-16

### Other

- fix mdbook directory not existing
- add github-pages workflow
- setup mdbook and some initial documentation
- create a readme
- *(dev)* make makers psql able to take more arguments
- *(actions)* add the release-binary workflow
- *(dev)* add commitlint and a commit-msg git hook
- Initial Rust project setup with axum, tracing, sqlx, etc. ([#2](https://github.com/bearbearean/nbsp/pull/2))
- Initialize Dependabot weekly update checks.
- Initialize release-plz GitHub Actions workflow.

## [0.0.0] - 2026-08-15

- Initial project creation.
