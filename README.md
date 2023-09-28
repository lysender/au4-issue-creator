# Issue Creator

Creates entries with fake data for testing purposes.

For the secret startup project.

## Usage

```shell
issue-creator --config path/to/config.toml COMMAND
```

### Commands

- create - Creates issues into the specified project in config file
- help - Displays help

## Config

```toml
token = "token"
base_url = "https://example.com/api"
project_id = "123"
issue_count = 10
```
