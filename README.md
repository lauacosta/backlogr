# Backlogr
**A work-in-progress CLI for interacting with the Taiga REST API.**

> ⚠️ **Personal Tool Warning**  
> `backlogr` is tailored for personal CI workflows and only implements what *I* need.  
> It is not a general-purpose client and may never be. Current limitations include:
> - Only supports User Stories (no tasks, issues, or epics)
> - Basic status transitions only (New → WIP → Done)
> - No bulk operations or advanced filtering
> - Limited error handling and validation

---

## 🚀 Quick Start

1. **Install backlogr** (see [Installation](#-installation))
2. **Set up environment variables**:
   ```sh
   export USERNAME=your_taiga_username
   export PASSWORD=your_taiga_password
   export PROJECT_NAME=your_project_name
   ```
3. **Create and manage stories**:
   ```sh
   # Create a new story
   backlogr create
   
   # List all stories
   backlogr list

   # Move story #10 to work in progress
   backlogr wip 10
   
   # Mark story #10 as done
   backlogr done 10

   # Delete story #10
   backlogr delete 10
   ```

---

## 📦 Installation

### From Releases (Recommended)
```sh
# Download the latest release for your platform
curl -L https://github.com/lauacosta/backlogr/releases/latest/download/backlogr-linux-x86_64.tar.gz | tar xz
sudo mv backlogr /usr/local/bin/
```

### From Source
```sh
# Requires Rust 1.85+
git clone https://github.com/lauacosta/backlogr.git
cd backlogr
cargo install --path .
```

---

## 📋 Prerequisites

- **Taiga Instance**: Access to a Taiga instance (tested with Taiga 6.x)
- **Account**: Valid Taiga username and password
- **Project**: Existing project in Taiga with User Stories enabled
- **Rust**: 1.85+ (if building from source)

---

## ✨ Features

- 🔐 Authenticate with Taiga using username/password
- 📝 Create new User Stories interactively
- 🔄 Transition stories between `New`, `WIP`, and `Done`
- 🗑️ Delete stories by title or ID
- 📊 List all stories grouped by status
- 🤖 Designed for **CI pipelines** and **manual project automation**
- 🌍 Environment variable support for secure credential handling

---

## 🛠️ Usage

### Basic Command Structure
```sh
backlogr --username <USERNAME> --password <PASSWORD> --project_name <PROJECT_NAME> [COMMAND]
```

### Using Environment Variables (Recommended)
```sh
export USERNAME=myuser
export PASSWORD=mypass
export PROJECT_NAME=myproject

backlogr [COMMAND]
```

### Command Help
```
▗▄▄▖ ▗▞▀▜▌▗▞▀▘█  ▄ █  ▄▄▄   ▄▄▄
▐▌ ▐▌▝▚▄▟▌▝▚▄▖█▄▀  █ █   █ █
▐▛▀▚▖         █ ▀▄ █ ▀▄▄▄▀ █
▐▙▄▞▘         █  █ █     ▗▄▖
                        ▐▌ ▐▌
                         ▝▀▜▌
                        ▐▙▄▞▘
    @lauacosta/backlogr 0.5.0

Usage: backlogr --username <USERNAME> --password <PASSWORD> --project_name <PROJECT_NAME> [COMMAND]

Commands:
  create  Creates a new User Story
  wip     Updates a User Story to 'In Progress'
  done    Updates a User Story to 'Done'
  delete  Deletes a User Story
  list    List User stories
  help    Print this message or the help of the given subcommand(s)

Options:
      --username <USERNAME>          Taiga Username [env: USERNAME=]
      --password <PASSWORD>          Taiga password [env: PASSWORD=]
      --project_name <PROJECT_NAME>  Taiga project name [env: PROJECT_NAME=]
  -h, --help                         Print help
  -V, --version                      Print version
```

---

## 📖 Command Examples

### Create a New Story
```sh
# Interactive creation
backlogr create --description "Implement user authentication" --description "Add JWT-based auth system"

# ✅ Created story: "Implement user authentication" (#42)
```

### List Stories
```sh
backlogr list

# Output:
# 📋 Total user stories: (8)
# 
# 🆕 New (2)
#   #41 Fix login bug
#   #43 Update documentation
# 
# 🔄 In Progress (1)  
#   #42 Implement user authentication
# 
# ✅ Done (5)
#   #40 Setup CI pipeline
#   #39 Initial project setup
#   ...
```

### Update Story Status
```sh
# Move to Work in Progress
backlogr wip 10
# ✅ Story #10 marked as 'In Progress'

# Mark as Done
backlogr done 15
# ✅ Story #15 marked as 'Done'
```

### Delete a Story
```sh
backlogr delete 32
# ✅ Successfully deleted user story (#32)
```

---

## 🤖 CI Pipeline Usage

### Environment Setup
```yaml
# GitHub Actions example
env:
  USERNAME: ${{ secrets.TAIGA_USERNAME }}
  PASSWORD: ${{ secrets.TAIGA_PASSWORD }}
  PROJECT_NAME: "MyProject"
```

### Common CI Workflows
```sh
# Create deployment story
backlogr create --subject "Deploy v$VERSION" --description "Automated deployment"

# ✅ Created story: "Deploy v1.0.1" (#42)

# Mark deployment as in progress
backlogr wip 42

# Mark as complete after successful deployment
backlogr done 42
```

### Exit Codes
- `0`: Success
- `1`: General error (authentication, network, etc.)
- `2`: Story not found
- `3`: Invalid project or permissions

---

## 🔧 Error Handling Examples

### Authentication Failure
```sh
backlogr list
# ❌ Authentication failed: HTTP 401: {"detail": "No active account found with the given credentials", "code": "invalid_credentials"}
# 💡 Troubleshooting authentication:
#    • Set environment variables:
#      export USERNAME=your_taiga_username
#      export PASSWORD=your_taiga_password
#    • Verify credentials by logging into Taiga web interface
#    • Check if your account is active and not locked
```

### Story Not Found
```sh
backlogr wip 50
# 🔍 Looking up user story with ref #50 in project...
# ❌ User story not found: User story with ref #50 not found.
# 💡 Story 'User story with ref #50 not found.' not found. Try:
#   • backlogr list           # See all available stories
#   • backlogr create         # Create a new story
#   • Check for typos in the story title
#   • Ensure you're in the correct project
```

### Network Issues
```sh
backlogr list
# ❌ Error: Failed to connect to Taiga instance. Please check your network connection.
# Exit code: 1
```

---

## 🏗️ Supported Taiga Versions

- **Tested**: Taiga 6.5.x, 6.6.x
- **Minimum**: Taiga 6.0+ (REST API v1)
- **Compatibility**: Should work with most modern Taiga instances

---

## 📄 License

MIT License - see [LICENSE](LICENSE) file for details.

---

## 🗺️ Roadmap

- [ ] JSON output format for better CI integration
- [ ] Bulk operations (create/update multiple stories)
- [ ] Story filtering and search
- [ ] Support for Tasks and Issues
- [ ] Custom field support
- [ ] Story templates

