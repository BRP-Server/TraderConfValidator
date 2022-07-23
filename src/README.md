# Trader config formatter

```
A tool to format DayZ trader config files

USAGE:
    trader_config_formatter.exe [OPTIONS] <file>

ARGS:
    <file>    Input: The file to be processed

OPTIONS:
    -d            Dry Run: If present the command will just check the file is valid
    -h, --help    Print help information
    -o            Output: The output file, by default overrides the input file
```

## Examples

**Just validate file:**
```powershell
trader_config_formatter.exe -d <Path to Trader\TraderConfig.txt>
```

**In place validation and formatting:**
```powershell
trader_config_formatter.exe <Path to Trader\TraderConfig.txt>
```

**Validate and output to different file:**
```powershell
trader_config_formatter.exe --output <Path to Output.txt> <Path to Trader\TraderConfig.txt>
```
