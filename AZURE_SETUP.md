# Azure App Registration Setup for PKCE

## Current Issue
The error `AADSTS70002: The provided request must include a 'client_secret' input parameter` occurs because the Azure App Registration is configured as a **Confidential Client** instead of a **Public Client**.

## Fix: Configure as Public Client

1. **Go to Azure Portal**: https://portal.azure.com
2. **Navigate to**: Azure Active Directory → App registrations
3. **Find your app**: `4c27a19f-a3d0-4cd2-8e05-9fd961f905df`
4. **Go to "Authentication"** section
5. **Under "Advanced settings"**:
   - ✅ **Enable "Allow public client flows"** 
   - ✅ **Set "Treat application as a public client"** to **Yes**
6. **Save the changes**

## Additional Configuration

### Redirect URIs
- Ensure `http://localhost:43110/callback` is added as a redirect URI
- Set platform type to **"Mobile and desktop applications"**

### API Permissions
Ensure these permissions are granted:
- `XboxLive.signin` (Xbox Live)
- `offline_access` (Microsoft Graph)

### Application Type
- **Application type**: Public client/native
- **Support account types**: Personal Microsoft accounts only

## Test After Changes
After making these changes, restart the application and try signing in again. The PKCE flow should work without requiring a client secret.

## Alternative: Device Code Flow
If the above doesn't work, we can implement Device Code Flow which is specifically designed for public clients and doesn't require PKCE.
