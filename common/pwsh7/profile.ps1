# TODO: find a way for dynamic path, this is not the right one for everyone.. Maybe locate dev-conf folder?
oh-my-posh init pwsh --config C:\shared\code\project\dev-conf\common\oh-my-posh\baba.omp.json | Invoke-Expression

function Update-EnvironmentVariables {
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")
}
