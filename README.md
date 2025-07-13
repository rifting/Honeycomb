<img width="1920" height="500" alt="image" src="https://github.com/user-attachments/assets/2f8802ed-45fc-413b-b5e1-a6a63e3b1b29" />

## An android device policy editor!
```
Usage: ./honeycomb [OPTIONS]

Options:
  -p, --policy-name <POLICY_NAME>  Name of policy you want to enable/disable
  -o, --out <OUT>                  Output file name
      --list-policies              List available policies and exit
      --overwrite                  Pass this argument to overwrite the original file
  -h, --help                       Print help
  -V, --version                    Print version
```
### Warning: Honeycomb is still in development. It may fail to correctly modify the device policy files. Always take backups if you're using the --overwrite argument

## Compilation
Follow the guide for cross compiling Rust to Android [here](https://mozilla.github.io/firefox-browser-architecture/experiments/2017-09-21-rust-on-android.html).

Honeycomb also works on Windows, Linux, and MacOS if you'd like to locally modify policies on a user profile file you already have.

## Credits
[rhythmcache](https://github.com/rhythmcache/) for their awesome [ABX converter & parser](https://github.com/rhythmcache/abx2xml-rs/)! 

## TODO
- Overwrite argument
- Allow the user to enter their own profile path instead of hardcoding it
- Add more safety checks
