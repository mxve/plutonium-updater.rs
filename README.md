# WIP

## Plutonium Updater CLI

### Features
- Multi-platform
- File hash checking
- Faster than [plutonium-updater-linux](https://github.com/mxve/plutonium-updater-linux)
  - File hash checking sped up x25
  - Download speed doubled
  - ["Benchmark"](https://screen.sbs/ia6lwg5sq)

### ToDo

- [ ] Replace clap to reduce dependencies
- [ ] Implement flags
  - [x] -d, --directory | Download directory
  - [ ] -f, --force | Force file hash re-check
  - [ ] -l, --launcher | Download launcher assets
  - [ ] -q, --quiet | Hide file actions
  - [ ] -s, --silent | Completely hide output
- [ ] Set exit codes
- [ ] Version checking without hash re-check