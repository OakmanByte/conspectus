## Github api tool in Rust ##

Tool can be run in two diffrent modes, either user or org. For user mode the config file will need the user_name field with the Githubname of the user you want to analyze the repositories for. For org mode it will need both the organization_name and the team_name(Github team) fields. Both modes will need a personal access token in the token field. Important note is that for org mode you will need to authorize the token for that specific organaization you want to read from as well.

Setup a config.ini file with the following format:

```
[Github]
//Reqiuired for both modes
token=******
//Required for user mode
user_name=****
//Required for org mode
organization_name=******
team_name******
```
