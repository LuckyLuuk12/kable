// SkinsService has been replaced by direct API usage
// 
// The skins functionality now focuses on Microsoft/Mojang account integration
// Use the API functions directly from '../api/skins' instead:
// - uploadSkinToAccount()
// - changeSkinModel() 
// - getCurrentSkinInfo()
// - selectSkinFile()
//
// This approach is cleaner and follows the modern pattern used elsewhere in the app.
//
// If complex skin management features are needed in the future (local skin storage,
// online galleries, etc.), they can be added to the backend skins module and
// exposed through additional API functions.

export class SkinsService {
  // Deprecated - use API functions directly
  static get deprecated() {
    console.warn('SkinsService is deprecated. Use ../api/skins functions directly. This file is kept because we might add more complex state to edit / list skins');
    return true;
  }
}
