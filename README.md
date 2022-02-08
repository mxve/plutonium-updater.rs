# WIP

## Plutonium Updater CLI

### Features
- Multi-platform
- File hash checking
- Faster than [plutonium-updater-linux](https://github.com/mxve/plutonium-updater-linux)
  - File hash checking sped up x25
  - Download speed doubled
  - ["Benchmark"](https://screen.sbs/ia6lwg5sq)

# Exit codes
- 0 success
- 101 error (rust panic)

### ToDo

~~- [ ] Replace clap to reduce dependencies~~
- [ ] Implement flags
  - [x] -d, --directory | Download directory
  - [x] -f, --force | Force file hash re-check
  - [x] -l, --launcher | Download launcher assets
  - [ ] -q, --quiet | Hide file actions
  - [ ] -s, --silent | Completely hide output
- [x] Set exit codes
- [x] Version checking without hash re-check