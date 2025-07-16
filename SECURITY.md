# Security Policy

## Reporting Security Vulnerabilities

We take the security of Kable seriously. If you discover a security vulnerability, please help us protect our users by reporting it responsibly.

### 🚨 Critical Security Issues

For **critical security vulnerabilities** that could:
- Compromise user accounts or credentials
- Allow unauthorized access to user systems
- Enable code execution or privilege escalation
- Expose sensitive user data

**Please report privately through:**
- **GitHub Security Advisories**: Use the "Report a vulnerability" button on our repository
- **Email**: Contact the repository maintainer directly (see GitHub profile)

### 📝 Non-Critical Security Issues

For **general security improvements** or **low-risk vulnerabilities**:
- Open a regular GitHub issue with the `security` label
- Provide detailed information about the potential issue
- Include steps to reproduce if applicable

## What to Include in Reports

### Required Information
- **Description** of the vulnerability
- **Steps to reproduce** the issue
- **Potential impact** and severity assessment
- **Affected versions** or components
- **Your environment** (OS, version, etc.)

### Optional but Helpful
- **Proof of concept** code or screenshots
- **Suggested fixes** or mitigations
- **References** to similar issues or CVEs

## Our Commitment

### Response Times
- **Critical vulnerabilities**: Response within 24-48 hours
- **High severity**: Response within 3-5 days
- **Medium/Low severity**: Response within 1-2 weeks

### Responsible Disclosure
- We will **acknowledge** your report promptly
- We will **investigate** thoroughly and provide updates
- We will **coordinate** on disclosure timing
- We will **credit** you in security advisories (if desired)

## Security Measures

### Application Security
- **Local-first architecture** - no sensitive data transmitted
- **Secure token storage** using OS credential managers  
- **Input validation** for all user inputs
- **Regular dependency updates** for security patches

### Development Security
- **Code review** process for all changes
- **Automated security scanning** in CI/CD
- **Dependency vulnerability monitoring**
- **Secure coding practices** and guidelines

### Infrastructure Security
- **GitHub repository security** features enabled
- **Branch protection** for main development branch
- **Required reviews** for sensitive changes
- **Signed releases** for authenticity verification

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | ✅ Active support  |
| < 0.1   | ❌ No support      |

**Note**: Only the latest release receives security updates. Users should upgrade to the latest version for security fixes.

## Security Best Practices for Users

### Account Security
- ✅ **Use strong passwords** for your Microsoft account
- ✅ **Enable two-factor authentication** on your Microsoft account
- ✅ **Keep Kable updated** to the latest version
- ✅ **Download only from official sources**

### System Security
- ✅ **Keep your OS updated** with security patches
- ✅ **Use reputable antivirus software**
- ✅ **Be cautious with mods** from unknown sources
- ✅ **Regular system backups** of important data

### Mod Security
- ✅ **Download mods from trusted sources** (CurseForge, Modrinth)
- ✅ **Verify mod authenticity** when possible
- ✅ **Read mod permissions** and requirements
- ❌ **Avoid suspicious or unverified mods**

## Known Security Considerations

### Authentication Tokens
- **Local storage**: Tokens are stored securely on your device
- **Automatic refresh**: Expired tokens are refreshed automatically
- **Secure transmission**: All auth communication uses HTTPS
- **Revocation**: Tokens can be revoked through Microsoft account settings

### Mod Loading
- **Sandboxing**: No additional sandboxing beyond Minecraft's security model
- **Verification**: Limited mod verification capabilities
- **User responsibility**: Users must verify mod safety
- **Isolation**: Mods run with Minecraft's permissions

### Network Security
- **Direct connections**: All API calls go directly from user's device
- **No proxying**: We don't proxy or intercept network traffic
- **HTTPS only**: All external communications use secure protocols
- **Certificate validation**: Proper SSL/TLS certificate validation

## Security Updates

### Update Process
1. **Security issue identified** and verified
2. **Fix developed** and tested thoroughly
3. **Security advisory** prepared (for public issues)
4. **Release published** with security fixes
5. **Users notified** through appropriate channels

### Update Notifications
- **Critical updates**: GitHub notifications and issue announcements
- **Regular updates**: Release notes and version changelogs
- **Security advisories**: GitHub Security Advisories system

## Third-Party Security

### Microsoft/Mojang Services
- **Authentication**: Uses official Microsoft OAuth 2.0 flows
- **API access**: Only uses public, documented APIs
- **No credential storage**: We never store or transmit your passwords
- **Compliance**: Follows Microsoft's security requirements

### Dependencies
- **Regular audits**: Automated dependency vulnerability scanning
- **Timely updates**: Security patches applied promptly
- **Minimal dependencies**: Limited dependency surface area
- **Trusted sources**: Only well-maintained, reputable packages

## Security FAQ

### Q: Is my Minecraft password safe?
**A:** Yes! Kable never sees your password. Authentication happens directly with Microsoft in your browser, and we only receive temporary access tokens.

### Q: What data does Kable access?
**A:** Only your Minecraft username, UUID, and authentication tokens needed for launching. See our [Privacy Policy](PRIVACY.md) for complete details.

### Q: Are my mods scanned for malware?
**A:** No, Kable doesn't scan mods for malware. Use trusted sources like CurseForge and Modrinth, and run your own antivirus scans on downloaded files.

### Q: Is it safe to use Kable on shared computers?
**A:** We recommend using Kable only on your personal devices. If you must use a shared computer, make sure to log out completely and clear any saved data.

### Q: What happens if my tokens are compromised?
**A:** Revoke the tokens through your Microsoft account security settings. Kable will automatically request new authentication when needed.

## Legal and Compliance

### Vulnerability Disclosure
- **Coordinated disclosure** preferred for security issues
- **Public disclosure** after reasonable time for fixes
- **Credit given** to security researchers (with permission)
- **No legal action** against good-faith security research

### Data Protection
- **No personal data collection** eliminates most privacy risks
- **Local data only** - no data breach risks from our servers
- **User control** over all authentication and configuration data
- **GDPR compliance** through data minimization

## Contact for Security Issues

### Primary Contact
- **GitHub Security Advisories**: Preferred method for vulnerability reports
- **Response time**: 24-48 hours for critical issues

### Alternative Contact
- **Repository issues**: For non-sensitive security discussions
- **Email**: Available through GitHub profile for urgent matters

---

**Thank you for helping to keep Kable secure for all users. Your responsible disclosure of security issues helps protect the entire community.**
