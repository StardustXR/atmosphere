# atmosphere
Adds an environment/background to Stardust XR!
> [!IMPORTANT]  
> Requires the [Stardust XR Server](https://github.com/StardustXR/server) to be running.

If you installed the Stardust XR server via:  
```note
sudo dnf group install stardust-xr
```
Or if you installed via the [installation script](https://github.com/cyberneticmelon/usefulscripts/blob/main/stardustxr_setup.sh), Atmosphere comes pre-installed

## How To Use
```sh
atmosphere install default_envs/the_grid # or any other folder with an env.kdl file inside it
atmosphere set-default the_grid
atmosphere show # the_grid implied since set to default
```


To make your own, follow the comments and structure of the `the_grid` environment in this repo.
