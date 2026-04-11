Describe 'MISE_TRUSTED_CONFIG_PATHS' {
    BeforeAll {
        $script:OriginalDir = Get-Location
        $script:TestRoot = Join-Path $TestDrive ([System.Guid]::NewGuid().ToString())
        New-Item -ItemType Directory -Path $script:TestRoot | Out-Null

        # Create two separate project directories with mise.toml files
        $script:DirA = Join-Path $script:TestRoot "project_a"
        $script:DirB = Join-Path $script:TestRoot "project_b"
        $script:DirC = Join-Path $script:TestRoot "project_c"
        New-Item -ItemType Directory -Path $script:DirA | Out-Null
        New-Item -ItemType Directory -Path $script:DirB | Out-Null
        New-Item -ItemType Directory -Path $script:DirC | Out-Null

        @"
[env]
PROJECT = "a"
"@ | Out-File (Join-Path $script:DirA ".mise.toml")

        @"
[env]
PROJECT = "b"
"@ | Out-File (Join-Path $script:DirB ".mise.toml")

        @"
[env]
PROJECT = "c"
"@ | Out-File (Join-Path $script:DirC ".mise.toml")
    }

    AfterAll {
        Set-Location $script:OriginalDir
        Remove-Item -Path $script:TestRoot -Recurse -Force -ErrorAction Ignore
        Remove-Item Env:MISE_TRUSTED_CONFIG_PATHS -ErrorAction Ignore
    }

    AfterEach {
        Remove-Item Env:MISE_TRUSTED_CONFIG_PATHS -ErrorAction Ignore
    }

    It 'trusts a single path set via env var' {
        $env:MISE_TRUSTED_CONFIG_PATHS = $script:DirA
        Set-Location $script:DirA
        $output = mise env | Out-String
        $output | Should -Match "export PROJECT=a"
    }

    It 'trusts multiple Windows paths separated by semicolon' {
        # On Windows, paths are separated by ; (not :) because absolute paths
        # contain : in the drive letter (e.g. C:\foo). Using ; avoids ambiguity.
        $env:MISE_TRUSTED_CONFIG_PATHS = "$($script:DirA);$($script:DirB)"
        Set-Location $script:DirA
        $output = mise env | Out-String
        $output | Should -Match "export PROJECT=a"
        Set-Location $script:DirB
        $output = mise env | Out-String
        $output | Should -Match "export PROJECT=b"
    }

    It 'does not trust a path not in the list' {
        # Only DirA is trusted; DirC is not in the list
        $env:MISE_TRUSTED_CONFIG_PATHS = $script:DirA
        Set-Location $script:DirC
        # mise should refuse to load an untrusted config and say so
        $output = mise env 2>&1 | Out-String
        $output | Should -Not -Match "export PROJECT=c"
        $output | Should -Match "trust"
    }

    It 'trusts nothing when env var is empty' {
        $env:MISE_TRUSTED_CONFIG_PATHS = ""
        Set-Location $script:DirA
        $output = mise env 2>&1 | Out-String
        $output | Should -Not -Match "export PROJECT=a"
        $output | Should -Match "trust"
    }
}
