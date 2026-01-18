<div align="center">

# backlogr

A minimal CLI for interacting with the Taiga REST API.

[![Crates.io](https://img.shields.io/crates/v/backlogr.svg)](https://crates.io/crates/backlogr)

</div>

For GitHub Actions usage, see [backlogr-action](https://github.com/lauacosta/backlogr-action).

`backlogr` is designed for personal CI workflows and implements only what is needed. It is not a general-purpose Taiga client.

**Limitations**
- User Stories only
- Status transitions: New → WIP → Done
- No bulk operations or filtering
- Minimal validation and error handling

## Quick Start

```sh
export USERNAME=your_username
export PASSWORD=your_password
export PROJECT_NAME=your_project
````

```sh
backlogr create
backlogr list
backlogr wip 10
backlogr done 10
backlogr delete 10
```

## Installation

### Binary

```sh
cargo binstall backlogr
```

### Crates.io

```sh
cargo install backlogr
```

### Releases

```sh
curl -L https://github.com/lauacosta/backlogr/releases/download/v0.0.1/backlogr-v0.0.1-x86_64-unknown-linux-musl.tar.gz | tar xz
sudo mv backlogr-v0.0.1-x86_64-unknown-linux-musl/backlogr /usr/local/bin/
```

### Source

```sh
git clone https://github.com/lauacosta/backlogr.git
cd backlogr
cargo install --path .
```

## Prerequisites

* Taiga 6.x instance
* Valid Taiga account
* Existing project with User Stories enabled
* Rust 1.85.1+ (source builds)

## Features

* Username/password authentication
* Interactive User Story creation
* Status transitions (New, WIP, Done)
* Story deletion by ID
* Story listing grouped by status
* Environment variable support for CI usage

## Usage

```sh
backlogr --username <USERNAME> --password <PASSWORD> --project_name <PROJECT_NAME> <COMMAND>
```

Environment variables are recommended.

### Commands

```
create   Create a user story
wip      Move story to In Progress
done     Move story to Done
delete   Delete a story
list     List stories
```

## Examples

### Create

```sh
backlogr create --subject "Implement auth" --description "JWT-based authentication"
```

### List

```sh
backlogr list
```

### Update

```sh
backlogr wip 10
backlogr done 10
```

### Delete

```sh
backlogr delete 32
```

## CI Usage

### GitHub Actions

```yaml
env:
  USERNAME: ${{ secrets.TAIGA_USERNAME }}
  PASSWORD: ${{ secrets.TAIGA_PASSWORD }}
  PROJECT_NAME: MyProject
```

### Example Workflow

```sh
backlogr create --subject "Deploy v1.0.1"
backlogr wip 42
backlogr done 42
```

### Commit-Based Automation

Supported commit format:

```
<modifier>: <message> (#<id>)
```

Modifier mapping:

* feat, fix, wip → WIP
* done → Done
* delete → Delete

See repository for the full script.

## Exit Codes

* 0: Success
* 1: General error
* 2: Story not found
* 3: Invalid project or permissions

## Error Handling

### Authentication

```sh
backlogr list
# Authentication failed (401)
```

### Story Not Found

```sh
backlogr wip 50
# User story not found
```

### Network Error

```sh
backlogr list
# Failed to connect to Taiga
```

## Supported Taiga Versions

* Tested: 6.5.x, 6.6.x
* Minimum: 6.0 (REST API v1)

## License

MIT. See LICENSE.

## Roadmap

* JSON output
* Bulk operations
* Filtering and search
* Tasks and Issues support
* Custom fields
* Story templates
