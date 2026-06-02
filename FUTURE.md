# What I want to do with this project in the future

## Restructure:
```
src/
├── main.rs                  % entry point, load .env, run lib
├── lib.rs                   % setup tauri, initialize things...
├── features/                % major features with subfeatures in rs files
│   ├── profiles/            % currently "KableInstallation" refactor to profiles
│   │   ├── kable_profile.rs 
│   │   ├── create.rs        % create kable profiles from scratch, 
│   │   │                      existing, mrpack, import, etc
│   │   ├── management.rs    % rename, delete, list profiles, etc
│   │   ├── mod.rs
│   │   └── advanced.rs      % jvm args, java version, etc
│   ├── launcher/            % manifest handling & launching per loader
│   │   ├── assets.rs        % handle assets
│   │   ├── vanilla.rs       % launch vanilla
│   │   ├── forge.rs         % launch forge
│   │   ├── fabric.rs        % launch fabric
│   │   ├── iris-fabric.rs   % launch iris-fabric
│   │   ├── quilt.rs         % launch quilt
│   │   └── neoforge.rs      % launch neoforge
│   ├── customization/       % app settings, themes, icons, sounds, etc.
│   │   ├── settings.rs      % app settings types / defaults, etc.
│   │   ├── icons.rs         % handle app icons
│   │   ├── sounds.rs        % handle app sounds
│   │   └── themes.rs        % handle app (css) themes
│   ├── resourcepacks/       % same as mods but for resourcepacks
│   ├── shaderpacks/         % same as mods but for shaderpacks
│   ├── worlds/              % local world management
│   └── mods/
│       ├── management.rs    % list/remove/disable/enable mods per profile 
│       ├── download.rs      % download, dependency resolution, etc.
│       ├── browser.rs       % given provider + query/options, browse mods
│       ├── metadata.rs      % store metadata about mods on-disk, just anything 
│       │                      available from provider + jar's manifest info. to 
│       │                      improve update checking
│       ├── update.rs        % check, apply updates to mods
│       ├── mod.rs
│       └── mrpack.rs        % parsing, handling, creating mrpack files
├── integrations/            % all of this is mostly procedural code that takes input  
│   │                          and gives output, no state/cache/fs writing
│   ├── curseforge/          
│   │   ├── model.rs         % types / structs for curseforge api
│   │   └── client.rs        % functional interface
│   ├── modrinth/            
│   │   ├── model.rs         % types / structs for modrinth api
│   │   └── client.rs        % functional interface
│   ├── minecraft/           % local .minecraft file serialization/handling
│   │   ├── versions.rs      % parsing of version json files, etc.
│   │   ├── accounts.rs      % parsing of launcher_accounts.json
│   │   ├── profiles.rs      % parsing of launcher_profiles.json
│   │   ├── skins.rs         % local skin files/base64 handling
│   ├── mojang-api/          % mojang api and handling
│   │   ├── auth.rs          % xbox/ms auth, token/cred handling, etc.
│   │   ├── skins.rs         % update, capes, etc. 
│   ├── discord/             % rich presence, maybe more later
│   │   ├── model.rs         % types / structs for discord api
│   │   ├── client.rs        % functional interface
├── system/                  
│   ├── symlinks.rs          % handling symlinks, junctions, etc. cross-platform 
│   ├── cache.rs             % some cache utilities (custom macro?)
│   ├── fs.rs                % (async) file system util
│   ├── net.rs               % network util
│   ├── java.rs              % java util, spawning, finding java on system, etc.
│   └── migration/           % code to migrate when necessary
│       └── v1_to_v2.rs      % example v1 to v2 migration.
├── api/                     
│   ├── model.rs             % types on which we typegen to TS for frontend
│   └── commands.rs          % all the #[tauri::command] functions we expose
```
## Refactor
Right now the code is a big mess. My goal is to refactor it like above... I might have to make separate crates of the api-types and keep a commands folder with per-feature files to allow for typegen to work easily.
We also might want to rework some of the custom logging logic and rename kable-macros crate to logging-macros or something like that.

## Add more features
- rename installation id's to something more user friendly, like "My Modpack" instead of "installation_1234"
- allow modpack creation from installations, very similar to "export installation" but it should make a valid `mrpack`
- add a way to organize/group mods,resourcepacks,etc in the UI for easier navigation. Maybe a tagging system or folders in the UI.
- add a "favorites" system for mods, resourcepacks, etc. so users can easily find their favorite ones.
- maybe see if we want to "recommend" various packs/mods?
- add a "news" section to show news about anything I like, if possible integrate minecraft official news, mods news, etc.
- make IntelliJ integration + plugin (perhaps?) to allow modders to easily test their mods through the launcher by running a specific run configuration that launches the mod in a test environment they could configure in the launcher / plugin.
- maybe we can make a "skin browser"
- skin editor.
- server manager tool: allow users to easily share their worlds / manage servers they locally or remotely host, maybe even integrate with hosting providers to allow users to easily rent servers and manage them through the launcher.
- figure out a correct/better way to "separate" installations, right now configs and modpacks might intefere with each other as we reuse the .minecraft folder instead of making a new one for every installation.
- a "auto mod update" feature per installation.
- maybe quick play for servers, modspacks, worlds, etc.
- config/options editor in-launcher for mods/minecraft itself per installation?
- server browser if there is a nice free api?
