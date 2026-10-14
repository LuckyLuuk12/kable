# Backend refactor:
Right now a backup of the old code is in `src-tauri/src-backup`, we will be refactoring the code in `src-tauri/src` to be more modular, maintainable and testable.

## Restructure:
```
src/
├── main.rs                  % entry point, load .env, run lib
├── lib.rs                   % setup tauri, initialize things...
├── features/                % major "app" features with subfeatures in rs files
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
│   │   ├── iris_fabric.rs   % launch iris-fabric
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
│   ├── mojang-api/          % mojang api and handling: https://minecraft.wiki/w/Mojang_API
│   │   ├── auth.rs          % xbox/ms auth, token/cred handling, etc.
│   │   ├── skins.rs         % update, capes, etc. 
│   ├── discord/             % rich presence, maybe more later
│   │   ├── model.rs         % types / structs for discord api
│   │   ├── client.rs        % functional interface
├── system/                  
│   ├── symlinks.rs          % handling symlinks, junctions, etc. cross-platform 
│   ├── cache.rs             % some cache utilities (custom macro?)
│   ├── fs.rs                % (async) file system util
│   ├── net.rs               % network util: http client, download util with progress, etc.
│   ├── java.rs              % java util, spawning, finding java on system, etc.
│   └── migration/           % code to migrate when necessary
│       └── v1_to_v2.rs      % example v1 to v2 migration.
├── api/                     % here we expose all the #[tauri::api] commands
│   └── <feature-command>.rs % all the #[tauri::command] functions we expose for each feature
```

## Refactor
1. rename `KableInstallation` to `KableProfile` (profiles are the instances and accounts are user "profiles/ xbox accounts")
2. define ALL structs in api-crates/<group>.rs files.
3. in all api/<commands>.rs files we only wrap existing functions and expose them as `#[tauri::command]`, so no additional logic in these files
4. most fs should be async.
5. use src-tauri/src-backup to look at older working code, keep src-tauri/src as our refactored code / clean.
6. there might be more features I forgot to show in this folder-tree, just group them logically or make new feature folders
7. clearly distinct between integrations, system and (app) features. integrations and system are almost not exposed to 
   frontend and just procedurally used in the features. Right now the src-backup has this all mixed up so decide yourself 
   when we can split something into a system or integration util function.
8. features should have little to no "tangling" (use functions from each other). 
   Each feature can and should use integrations and system functions though.
9. never use `#[tauri::command]` in files other than `api/<[sub]feature>.rs` files.
10. whenever a function from old code uses nested functions or perform checks/util itself try to refactor into smaller functions and put them in the correct place (system, integration, or feature util) so we can reuse them and keep the code clean.
11. "installations" are called "profiles" now, older "player/mojang profiles" are "accounts" now, we should reflect this in naming of functions, variables, etc. to avoid confusion.

# Frontend refactor:
Intention was to do this after the full backend refactor but it is hard to tell what is correct/missing/wrong without the frontend working so we do this in parallel.

## Basic structure:
```
src/
├── lib/
│   ├── actions/             % app-specific interaction wrapper utils like playing sound,.. might move this to services
│   ├── components/          % svelte components, grouped by feature
│   │   ├── auth/            % authentication UI
│   │   ├── mods/            % mod management UI
│   │   ├── notifications/   % notification UI
│   │   ├── resourcepacks/   % resourcepack management UI
│   │   ├── settings/        % settings UI
│   │   ├── shaders/         % shader management UI
│   │   ├── skins/           % skin management UI
│   │   └── profiles/        % profile management UI
│   ├── services/            % the "backend" of the frontend, all the logic that interacts with the backend and provides data to the UI 
│   │                        % this is the last layer before UI, components and routes should call this and not handle logic themselves!! 
│   ├── styles/              % full custom scss style organized with _<name>.scss files
│   │   └── variables/       % contains various subfiles for variables like colors, fonts, etc. (uses css variables in scss variables 
│   │                        % so we can use scss variables in code while keeping runtime modifiable css variables)
│   ├── utils/               % small util functions, AVOID using this for things that could have been in a service...
│   ├── api.ts               % generated api file from backend containing types AND a commands const object with api-functions
│   └── wrapped_api.ts       % wraps api.ts functions with the utils/result.ts code to more easily handle the generated commands 
└── routes/                  % svelte routes, grouped by feature
```

## Refactor
Similarly to the backend refactor we intended to perform first, we have to refactor the frontend accordingly. The main goal is to have a clear separation of concerns and to make the code more maintainable and testable by:
1. clearly separating the UI components from the logic that interacts with the backend (services)
2. grouping components and services by feature
3. avoiding "utils" that contain logic that should be in a service
4. having svelte 5 idiomatic code (e.g. no stores but `$state` inside services, no `on:click` but `onclick` for handlers/attributes, using `$derived` or instead of `$:` or `$someStore`, etc)
5. Keep styling centralized in `styles` folder, forbidden to use inline-styling unless absolutely necessary (e.g. dynamic styles that cannot be done in scss), and avoid using css frameworks! Also keep the styling organized
6. Keep svelte files ordered by: 
   - (optional) `<!-- @component -->` comment at the top of the file
   - `<script>`   (with `lang="ts"`)
   - `<template>` (html)
   - `<style>`    (with `lang="scss"`)
7. Never manually modify `api.ts` or `wrapped_api.ts`, they are generated from the backend code and will be overwritten on build.
8. During the refactor, if some backend function has not (yet) been refactored just "stub" the frontend service call (e.g. make an empty (async) function in the relevant service and use that in the component/route) so we can still work on the frontend while the backend is being refactored. Once the backend function is refactored, we can implement the service call properly.
9. We changed "installations" to "profiles" in backend and we should reflect this in the frontend, for ease it is also better to keep variable naming consistenent (e.g. don't use `activeAccount`, `activeProfile`, `currentAccount` interchangeably, as well as `currentInstallation` or `currentProfile` etc. but rather use `activeProfile` and `activeAccount` consistently throughout the codebase).
10. We changed "player/mojang profiles" to "accounts" in backend and we should reflect this in the frontend, for ease it is also better to keep variable naming consistenent (e.g. don't use `activeAccount`, `activeProfile`, `currentAccount` interchangeably, as well as `currentInstallation` or `currentProfile` etc. but rather use `activeProfile` and `activeAccount` consistently throughout the codebase).
11. we use barrel (re) exporting so in each (sub) folder we have an `index.ts` file that exports all the files in that folder
12. since we use barrel exporting, whenever some frontend file imports something it should always do `import { something, type sometype } from "$lib"` and group it all under that single import. Only (to avoid cycling imports) if the file is in the same folder as the imported file, we can do `import { something, type sometype } from "./<file>"` instead of `$lib`.
13. some components might have to change significantly since we (more smartly) made new types for things like Modrinth projects which contain all "metadata" 



# More future ideas:
## Add more features (NOT before the refactor!!)
- rename profile id's to something more user friendly, like "My Modpack" instead of "profile_1234"
- allow modpack creation from profiles, very similar to "export profile" but it should make a valid `mrpack`
- add a way to organize/group mods,resourcepacks,etc in the UI for easier navigation. Maybe a tagging system or folders in the UI.
- add a "favorites" system for mods, resourcepacks, etc. so users can easily find their favorite ones.
- maybe see if we want to "recommend" various packs/mods?
- add a "news" section to show news about anything I like, if possible integrate minecraft official news, mods news, etc.
- make IntelliJ integration + plugin (perhaps?) to allow modders to easily test their mods through the launcher by running a specific run configuration that launches the mod in a test environment they could configure in the launcher / plugin.
- maybe we can make a "skin browser"
- skin editor.
- server manager tool: allow users to easily share their worlds / manage servers they locally or remotely host, maybe even integrate with hosting providers to allow users to easily rent servers and manage them through the launcher.
- figure out a correct/better way to "separate" profiles, right now configs and modpacks might intefere with each other as we reuse the .minecraft folder instead of making a new one for every profile.
- a "auto mod update" feature per profile.
- maybe quick play for servers, modspacks, worlds, etc.
- config/options editor in-launcher for mods/minecraft itself per profile?
- server browser if there is a nice free api?