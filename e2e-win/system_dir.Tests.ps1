Describe 'system_dir' {
    BeforeAll {
        $script:originalPwd = Get-Location
        $script:originalProgramData = $env:PROGRAMDATA
        Set-Location $TestDrive
        $env:MISE_TRUSTED_CONFIG_PATHS = $TestDrive
    }
    AfterAll {
        Set-Location $script:originalPwd
        $env:PROGRAMDATA = $script:originalProgramData
        Remove-Item -Path Env:\MISE_TRUSTED_CONFIG_PATHS -ErrorAction SilentlyContinue
        Remove-Item -Path Env:\MISE_SYSTEM_CONFIG_DIR    -ErrorAction SilentlyContinue
        Remove-Item -Path Env:\MISE_SYSTEM_DATA_DIR      -ErrorAction SilentlyContinue
    }

    It 'loads system config when MISE_SYSTEM_CONFIG_DIR is explicitly set' {
        $sysDir = Join-Path $TestDrive "sys_config"
        New-Item -ItemType Directory -Path $sysDir -Force | Out-Null
        @"
[env]
MISE_TEST_SYSTEM_VAR = "from_system"
"@ | Out-File (Join-Path $sysDir "config.toml") -Encoding utf8NoBOM
        $env:MISE_SYSTEM_CONFIG_DIR = $sysDir
        $output = mise env 2>&1 | Out-String
        $output | Should -Match "export MISE_TEST_SYSTEM_VAR=from_system"
        Remove-Item -Path Env:\MISE_SYSTEM_CONFIG_DIR
    }

    It 'does not error when ProgramData\mise does not exist' {
        $fakeProgData = Join-Path $TestDrive "fake_pgdata1"
        New-Item -ItemType Directory -Path $fakeProgData -Force | Out-Null
        $env:PROGRAMDATA = $fakeProgData
        try {
            { mise version | Out-Null } | Should -Not -Throw
        } finally {
            $env:PROGRAMDATA = $script:originalProgramData
        }
    }

    It 'warns and skips system config when ProgramData\mise is owned by non-privileged user' {
        $fakeProgData = Join-Path $TestDrive "fake_pgdata2"
        $fakeMiseDir  = Join-Path $fakeProgData "mise"
        New-Item -ItemType Directory -Path $fakeMiseDir -Force | Out-Null
        @"
[env]
MISE_SHOULD_NOT_APPEAR = "yes"
"@ | Out-File (Join-Path $fakeMiseDir "config.toml") -Encoding utf8NoBOM
        $env:PROGRAMDATA = $fakeProgData
        try {
            $output = mise env 2>&1 | Out-String
            $output | Should -Not -Match "MISE_SHOULD_NOT_APPEAR"
            $output | Should -Match "owned by BUILTIN"
        } finally {
            $env:PROGRAMDATA = $script:originalProgramData
        }
    }
}
