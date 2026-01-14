//! Windows admin ownership verification for system directories.
//!
//! This module provides functionality to verify that the system directory (ProgramData/mise)
//! is owned by an administrator-level account, preventing non-admin users from poisoning
//! the system configuration.
//!
//! On Windows, unlike Unix, non-admin users can create directories in ProgramData.
//! To prevent one user from "poisoning" the system config for other users, we verify
//! that the directory owner is a trusted system account (SYSTEM, Administrators, or TrustedInstaller).

#[cfg(windows)]
mod windows_impl {
    use std::path::Path;
    use std::ptr;

    use eyre::{Result, eyre};
    use winapi::shared::minwindef::DWORD;
    use winapi::shared::winerror::ERROR_SUCCESS;
    use winapi::um::accctrl::SE_FILE_OBJECT;
    use winapi::um::aclapi::GetNamedSecurityInfoW;
    use winapi::um::errhandlingapi::GetLastError;
    use winapi::um::securitybaseapi::{
        AllocateAndInitializeSid, EqualSid, FreeSid, IsWellKnownSid,
    };
    use winapi::um::winbase::LocalFree;
    use winapi::um::winnt::{
        OWNER_SECURITY_INFORMATION, PSID, SECURITY_DESCRIPTOR, SID_IDENTIFIER_AUTHORITY,
        WinBuiltinAdministratorsSid, WinLocalSystemSid,
    };

    // SECURITY_NT_AUTHORITY = {0,0,0,0,0,5}
    const SECURITY_NT_AUTHORITY: SID_IDENTIFIER_AUTHORITY = SID_IDENTIFIER_AUTHORITY {
        Value: [0, 0, 0, 0, 0, 5],
    };

    // TrustedInstaller is NT SERVICE\TrustedInstaller
    // Its RID is 80 (SECURITY_SERVICE_ID_BASE_RID) followed by specific sub-authorities
    // The full SID is S-1-5-80-956008885-3418522649-1831038044-1853292631-2271478464
    const SECURITY_SERVICE_ID_BASE_RID: DWORD = 80;
    const TRUSTED_INSTALLER_RID1: DWORD = 956008885;
    const TRUSTED_INSTALLER_RID2: DWORD = 3418522649;
    const TRUSTED_INSTALLER_RID3: DWORD = 1831038044;
    const TRUSTED_INSTALLER_RID4: DWORD = 1853292631;
    const TRUSTED_INSTALLER_RID5: DWORD = 2271478464;

    /// Check if a path is owned by an admin-level account.
    ///
    /// Returns Ok(true) if the path is owned by:
    /// - SYSTEM (LocalSystem)
    /// - Administrators group
    /// - TrustedInstaller
    ///
    /// Returns Ok(false) if owned by a non-admin account.
    /// Returns Err if the ownership cannot be determined (e.g., path doesn't exist, access denied).
    pub fn is_admin_owned(path: &Path) -> Result<bool> {
        // Convert path to wide string for Windows API
        let wide_path: Vec<u16> = path
            .to_string_lossy()
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        let mut owner_sid: PSID = ptr::null_mut();
        let mut security_descriptor: *mut SECURITY_DESCRIPTOR = ptr::null_mut();

        // Get the owner SID from the security descriptor
        let result = unsafe {
            GetNamedSecurityInfoW(
                wide_path.as_ptr(),
                SE_FILE_OBJECT,
                OWNER_SECURITY_INFORMATION,
                &mut owner_sid,
                ptr::null_mut(), // group
                ptr::null_mut(), // dacl
                ptr::null_mut(), // sacl
                &mut security_descriptor as *mut _ as *mut _,
            )
        };

        if result != ERROR_SUCCESS {
            let error_code = unsafe { GetLastError() };
            return Err(eyre!(
                "Failed to get security info for {}: error code {}",
                path.display(),
                error_code
            ));
        }

        // Check ownership and free security descriptor
        let is_admin = check_admin_ownership(owner_sid, path);

        // Free the security descriptor allocated by GetNamedSecurityInfoW
        if !security_descriptor.is_null() {
            unsafe {
                LocalFree(security_descriptor as *mut _);
            }
        }

        is_admin
    }

    fn check_admin_ownership(owner_sid: PSID, path: &Path) -> Result<bool> {
        if owner_sid.is_null() {
            return Err(eyre!(
                "Failed to get owner SID for {}",
                path.display()
            ));
        }

        // Check if owner is LocalSystem
        if unsafe { IsWellKnownSid(owner_sid, WinLocalSystemSid) } != 0 {
            debug!("Path {} is owned by LocalSystem", path.display());
            return Ok(true);
        }

        // Check if owner is Administrators group
        if unsafe { IsWellKnownSid(owner_sid, WinBuiltinAdministratorsSid) } != 0 {
            debug!("Path {} is owned by Administrators", path.display());
            return Ok(true);
        }

        // Check if owner is TrustedInstaller
        if is_trusted_installer(owner_sid)? {
            debug!("Path {} is owned by TrustedInstaller", path.display());
            return Ok(true);
        }

        debug!(
            "Path {} is NOT owned by an admin account",
            path.display()
        );
        Ok(false)
    }

    /// Check if a SID matches TrustedInstaller
    fn is_trusted_installer(sid: PSID) -> Result<bool> {
        let mut trusted_installer_sid: PSID = ptr::null_mut();
        let mut authority = SECURITY_NT_AUTHORITY;

        // Create the TrustedInstaller SID
        // S-1-5-80-956008885-3418522649-1831038044-1853292631-2271478464
        let result = unsafe {
            AllocateAndInitializeSid(
                &mut authority,
                6, // 6 sub-authorities
                SECURITY_SERVICE_ID_BASE_RID,
                TRUSTED_INSTALLER_RID1,
                TRUSTED_INSTALLER_RID2,
                TRUSTED_INSTALLER_RID3,
                TRUSTED_INSTALLER_RID4,
                TRUSTED_INSTALLER_RID5,
                0,
                0,
                &mut trusted_installer_sid,
            )
        };

        if result == 0 {
            return Err(eyre!("Failed to create TrustedInstaller SID"));
        }

        let is_equal = unsafe { EqualSid(sid, trusted_installer_sid) } != 0;

        // Free the allocated SID
        if !trusted_installer_sid.is_null() {
            unsafe {
                FreeSid(trusted_installer_sid);
            }
        }

        Ok(is_equal)
    }

    /// Verify the system directory is admin-owned.
    ///
    /// This function checks if the mise system directory (typically ProgramData\mise)
    /// is owned by an admin account. This is a security measure to prevent
    /// non-admin users from creating a malicious system directory that would
    /// affect all users on the machine.
    ///
    /// Returns:
    /// - Ok(true) if the directory is admin-owned or doesn't exist (safe to use once created by admin)
    /// - Ok(false) if the directory exists but is NOT admin-owned (potential security risk)
    /// - Err on other errors (e.g., access denied)
    pub fn verify_system_dir_ownership(path: &Path) -> Result<bool> {
        if !path.exists() {
            // Directory doesn't exist yet - this is fine, it will be created
            // with proper permissions when needed
            debug!(
                "System directory {} does not exist (will be created as needed)",
                path.display()
            );
            return Ok(true);
        }

        is_admin_owned(path)
    }
}

#[cfg(windows)]
pub use windows_impl::*;

// These functions are used in tests on non-Windows platforms
#[cfg(not(windows))]
#[allow(dead_code)]
pub fn verify_system_dir_ownership(_path: &std::path::Path) -> eyre::Result<bool> {
    // On Unix, we trust system directories by convention
    // (/etc is typically protected by filesystem permissions)
    Ok(true)
}

#[cfg(not(windows))]
#[allow(dead_code)]
pub fn is_admin_owned(_path: &std::path::Path) -> eyre::Result<bool> {
    // On Unix, this concept doesn't apply in the same way
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(not(windows))]
    fn test_unix_always_trusted() {
        use std::path::Path;
        assert!(verify_system_dir_ownership(Path::new("/etc")).unwrap());
        assert!(is_admin_owned(Path::new("/etc")).unwrap());
    }

    // Note: Windows-specific tests are challenging because:
    // 1. Tests typically run as non-admin
    // 2. Creating admin-owned directories requires elevation
    // 3. Changing directory ownership requires admin privileges
    //
    // E2E tests for this feature focus on:
    // - Testing that the setting toggle works
    // - Testing warning messages when verification fails
    // - Testing MISE_SYSTEM_DIR override behavior
}
