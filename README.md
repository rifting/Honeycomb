<img width="1920" height="500" alt="image" src="https://github.com/user-attachments/assets/2f8802ed-45fc-413b-b5e1-a6a63e3b1b29" />

## An android device policy editor!
```
Usage: ./honeycomb [OPTIONS]

Options:
  -p, --policy-name <POLICY_NAME>    Name of policy you want to enable/disable
      --profile-path <PROFILE_PATH>  Input file. For the primary user on android devices, this is typically /data/system/users/0.xml [default: /data/system/users/0.xml]
  -o, --out <OUT>                    Output file name
      --list-policies                List available policies and exit
      --overwrite                    Pass this argument to overwrite the original file
  -h, --help                         Print help
  -V, --version                      Print version
```
### Warning: Honeycomb is still in development. It may fail to correctly modify the device policy files. Always take backups if you're using the --overwrite argument

### Example Usage

#### Listing Enabled Policies
```
> ./honeycomb --list-policies

no_install_unknown_sources
no_factory_reset
no_config_location
no_add_clone_profile
no_safe_boot
no_config_credentials
no_config_date_time
```

#### Removing Policies
```
> ./honeycomb --policy-name no_install_unknown_sources --out out.xml

REMOVING the no_install_unknown_sources policy

Found no_install_unknown_sources with start offset 336 and end offset 367
Successfully disabled the no_install_unknown_sources policy
Wrote XML without policy to out.xml!
```
#### Creating Policies
```
> ./honeycomb --policy-name no_install_unknown_sources --out out.xml

CREATING the no_install_unknown_sources policy
Successfully added the no_install_unknown_sources policy

Wrote XML with the new policy to out.xml!
```

## Compilation
Follow the guide for cross compiling Rust to Android [here](https://mozilla.github.io/firefox-browser-architecture/experiments/2017-09-21-rust-on-android.html).

Honeycomb also works on Windows, Linux, and MacOS if you'd like to locally modify policies on a user profile file you already have.

## Credits
[rhythmcache](https://github.com/rhythmcache/) for their awesome [ABX converter & parser](https://github.com/rhythmcache/abx2xml-rs/)! 

## TODO
- Overwrite argument
- Allow the user to enter their own profile path instead of hardcoding it
- Add more safety checks
