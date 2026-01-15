Describe 'config_path_separators' {
    BeforeAll {
        # Create test directory structure
        $TestRoot = Get-Location
        $Project1 = Join-Path $TestRoot "project1"
        $Project2 = Join-Path $TestRoot "project2"
        $Project3 = Join-Path $TestRoot "project3"
        
        New-Item -ItemType Directory -Path $Project1 -Force | Out-Null
        New-Item -ItemType Directory -Path $Project2 -Force | Out-Null
        New-Item -ItemType Directory -Path $Project3 -Force | Out-Null
    }

    AfterAll {
        # Clean up test directories
        Set-Location $TestRoot
        Remove-Item -Path $Project1 -Recurse -Force -ErrorAction Ignore
        Remove-Item -Path $Project2 -Recurse -Force -ErrorAction Ignore
        Remove-Item -Path $Project3 -Recurse -Force -ErrorAction Ignore
        Remove-Item Env:MISE_CEILING_PATHS -ErrorAction Ignore
        Remove-Item Env:MISE_IGNORED_CONFIG_PATHS -ErrorAction Ignore
        Remove-Item Env:MISE_TASK_DISABLE_PATHS -ErrorAction Ignore
        Remove-Item Env:MISE_TRUSTED_CONFIG_PATHS -ErrorAction Ignore
    }

    It 'MISE_CEILING_PATHS uses semicolon separator on Windows' {
        # Create configs in both projects
        @"
[env]
PROJECT1 = "true"
"@ | Out-File (Join-Path $Project1 ".mise.toml")

        @"
[env]
PROJECT2 = "true"
"@ | Out-File (Join-Path $Project2 ".mise.toml")

        Set-Location $Project1
        
        # Set multiple ceiling paths using SEMICOLON separator (Windows)
        $env:MISE_CEILING_PATHS = "$Project1;$Project2"
        $output = mise env | Out-String
        $output | Should -Match "export PROJECT1=true"
        
        Remove-Item Env:MISE_CEILING_PATHS -ErrorAction Ignore
    }

    It 'MISE_IGNORED_CONFIG_PATHS uses semicolon separator on Windows' {
        $Ignored = Join-Path $TestRoot "ignored"
        $NotIgnored = Join-Path $TestRoot "not_ignored"
        
        New-Item -ItemType Directory -Path $Ignored -Force | Out-Null
        New-Item -ItemType Directory -Path $NotIgnored -Force | Out-Null
        
        @"
[env]
IGNORED = "should_not_appear"
"@ | Out-File (Join-Path $Ignored ".mise.toml")

        @"
[env]
NOT_IGNORED = "true"
"@ | Out-File (Join-Path $NotIgnored ".mise.toml")

        # Use absolute paths with SEMICOLON separator (Windows)
        $IgnoredConfig = Join-Path $Ignored ".mise.toml"
        $Project1Config = Join-Path $Project1 ".mise.toml"
        $env:MISE_IGNORED_CONFIG_PATHS = "$IgnoredConfig;$Project1Config"
        
        Set-Location $Ignored
        $output = mise env | Out-String
        $output | Should -Not -Match "export IGNORED="
        
        Remove-Item Env:MISE_IGNORED_CONFIG_PATHS -ErrorAction Ignore
        Remove-Item -Path $Ignored -Recurse -Force -ErrorAction Ignore
        Remove-Item -Path $NotIgnored -Recurse -Force -ErrorAction Ignore
    }

    It 'MISE_TASK_DISABLE_PATHS uses semicolon separator on Windows' {
        $Tasks1 = Join-Path $TestRoot "tasks1"
        $Tasks2 = Join-Path $TestRoot "tasks2"
        $Tasks1Dir = Join-Path $Tasks1 ".mise\tasks"
        $Tasks2Dir = Join-Path $Tasks2 ".mise\tasks"
        
        New-Item -ItemType Directory -Path $Tasks1Dir -Force | Out-Null
        New-Item -ItemType Directory -Path $Tasks2Dir -Force | Out-Null

        @"
#!/usr/bin/env bash
echo "task1"
"@ | Out-File (Join-Path $Tasks1Dir "test.sh")

        @"
#!/usr/bin/env bash
echo "task2"
"@ | Out-File (Join-Path $Tasks2Dir "test2.sh")

        Set-Location $Tasks1
        
        # Disable tasks using SEMICOLON separator (Windows)
        $env:MISE_TASK_DISABLE_PATHS = "$Tasks1;$Tasks2"
        $output = mise tasks --no-header | Out-String
        $output | Should -Not -Match "test"
        
        Remove-Item Env:MISE_TASK_DISABLE_PATHS -ErrorAction Ignore
        Remove-Item -Path $Tasks1 -Recurse -Force -ErrorAction Ignore
        Remove-Item -Path $Tasks2 -Recurse -Force -ErrorAction Ignore
    }

    It 'MISE_TRUSTED_CONFIG_PATHS uses semicolon separator on Windows' {
        $Trusted1 = Join-Path $TestRoot "trusted1"
        $Trusted2 = Join-Path $TestRoot "trusted2"
        
        New-Item -ItemType Directory -Path $Trusted1 -Force | Out-Null
        New-Item -ItemType Directory -Path $Trusted2 -Force | Out-Null

        @"
[env]
TRUSTED1 = "value"
"@ | Out-File (Join-Path $Trusted1 "mise.toml")

        @"
[env]
TRUSTED2 = "value"
"@ | Out-File (Join-Path $Trusted2 "mise.toml")

        # Set multiple trusted paths using SEMICOLON separator (Windows)
        $env:MISE_TRUSTED_CONFIG_PATHS = "$Trusted1;$Trusted2"
        
        Set-Location $Trusted1
        $output = mise env | Out-String
        $output | Should -Match "export TRUSTED1=value"
        
        Remove-Item Env:MISE_TRUSTED_CONFIG_PATHS -ErrorAction Ignore
        Remove-Item -Path $Trusted1 -Recurse -Force -ErrorAction Ignore
        Remove-Item -Path $Trusted2 -Recurse -Force -ErrorAction Ignore
    }

    It 'handles Windows drive letters correctly with semicolon separator' {
        # This test verifies that C:\path;D:\path is parsed as two paths
        # not three tokens (C, \path:D, \path)
        
        # Create test directories with drive letters (if on Windows)
        $Drive1Path = Join-Path $TestRoot "drive_test1"
        $Drive2Path = Join-Path $TestRoot "drive_test2"
        
        New-Item -ItemType Directory -Path $Drive1Path -Force | Out-Null
        New-Item -ItemType Directory -Path $Drive2Path -Force | Out-Null

        @"
[env]
DRIVE1 = "true"
"@ | Out-File (Join-Path $Drive1Path ".mise.toml")

        # Use full paths which will include drive letters on Windows
        $env:MISE_CEILING_PATHS = "$Drive1Path;$Drive2Path"
        
        Set-Location $Drive1Path
        $output = mise env | Out-String
        $output | Should -Match "export DRIVE1=true"
        
        Remove-Item Env:MISE_CEILING_PATHS -ErrorAction Ignore
        Remove-Item -Path $Drive1Path -Recurse -Force -ErrorAction Ignore
        Remove-Item -Path $Drive2Path -Recurse -Force -ErrorAction Ignore
    }
}
