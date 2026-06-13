// This module contains code related to interacting with Mojang's APIs, as documented here: https://minecraft.wiki/w/Mojang_API
// This includes oauth, skins, and anything that is NOT directly/only related to just parsing the .minecraft directory, see src/integrations/minecraft.
pub mod auth;
pub mod avatar;
pub mod skins;
