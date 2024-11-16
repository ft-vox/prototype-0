# Function to recursively process directories
function Do-Work {
    param (
        [string]$Directory
    )

    # Get all subdirectories
    Get-ChildItem -Path $Directory -Directory | ForEach-Object {
        $currentDir = $_.FullName
        $compileScript = Join-Path $currentDir "print_compile_commands.bat"

        if (Test-Path $compileScript) {
            # Execute the batch file and capture its output
            Push-Location $currentDir
            $output = & $compileScript
            Pop-Location

            # If output is not empty, add it to our results
            if ($output) {
                $output
            }
        } else {
            # Recurse into subdirectories
            Do-Work -Directory $currentDir
        }
    }
}

# Execute and create JSON file
$results = @(
    # Start recursive processing from lib directory
    Do-Work -Directory "lib"
)

# Convert to JSON and save to file
# ConvertTo-Json will handle array formatting properly
$results | ConvertTo-Json -Depth 100 > compile_commands.json
