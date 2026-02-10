flowchart TB
    %% --- Styles ---
    classDef disk fill:#eee,stroke:#333,stroke-width:2px,stroke-dasharray: 5 5;
    classDef cPacker fill:#ffcccc,stroke:#cc0000,stroke-width:2px;
    classDef rustLoader fill:#fff2cc,stroke:#d6b656,stroke-width:2px;
    classDef targetProcess fill:#d5e8d4,stroke:#82b366,stroke-width:2px;
    classDef c2 fill:#dae8fc,stroke:#6c8ebf,stroke-width:2px;
    classDef data fill:#f8cecc,stroke:#b85450,stroke-dasharray: 3 3;

    %% --- C2 Infrastructure (Top Layer) ---
    subgraph C2_Infra ["🌐 Infrastructure C2 & Opérateur"]
        direction LR
        Operator["👤 Opérateur"]
        DiscordAPI["💬 Discord API (v10)\n(HTTPS / TLS)"]
        Operator <-->|"Commandes (shell, scr, !loaddll) / Résultats"| DiscordAPI
    end
    class C2_Infra c2

    %% --- Target Machine ---
    subgraph TargetMachine ["💻 Machine Cible (Windows x64)"]
        direction TB

        %% --- Stage 0: On Disk ---
        subgraph Disk ["💾 Sur Disque"]
            PackedBinary["📦 Packed Binary.exe\n(Point d'entrée initial)"]:::disk
            subgraph BinaryContents ["Contenu du Binaire"]
                CStub["🛠️ Stub Packer C\n(Pure C, No-CRT)"]:::cPacker
                EncryptedRustLoader["🔒 Payload Chiffré/Compressé\n(contient loader.exe Rust)"]:::data
            end
        end

        %% --- Stage 1 & 2: Initial Execution Process ---
        subgraph InitialProcess ["⚙️ Processus Initial (RAM)"]
            direction TB

            %% --- Stage 1: C Packer Execution ---
            subgraph Stage1_CPacker ["Phase 1 : Exécution du Packer C (No-CRT)"]
                direction TB
                StartC["▶️ Démarrage du Stub C"]
                PEBWalk["🚶‍♂️ Walking PEB -> Ldr -> InLoadOrder\n(Trouver kernel32/ntdll sans import)"]
                ResolveAPIs["🔍 Résolution Manuelle d'APIs\n(par hachage de noms)"]
                DecryptDecompress["🔓 Déchiffrement & Décompression\ndu Payload Rust"]
                ManualMapLocal["🗺️ Manual Mapping Local\n(Map sections du PE Rust en mémoire)"]
                HandleTLS["⚙️ Exécution Manuelle TLS Callbacks\n(Crucial pour init runtime Rust)"]
                JmpEntryPoint["↪️ JMP vers EntryPoint Rust"]

                StartC --> PEBWalk --> ResolveAPIs --> DecryptDecompress --> ManualMapLocal --> HandleTLS --> JmpEntryPoint
            end
            class Stage1_CPacker cPacker

            %% --- Transition Link ---
            JmpEntryPoint -.->|"Transfert d'exécution"| StartRust

            %% --- Stage 2: Rust Loader Execution ---
            subgraph Stage2_RustLoader ["Phase 2 : Loader Rust Intermédiaire"]
                direction TB
                StartRust["▶️ Démarrage loader.exe (Rust)"]
                noteLoader["Contient beacon.dll chiffré (XOR) en statique"]
                
                subgraph IndirectSyscalls ["🛡️ Mécanisme Indirect Syscalls"]
                    ObfHash["#️⃣ Hachage DJB2 (Compile-time)"]
                    ResolveSSN["🕵️ Résolution SSN via PEB/Ntdll"]
                    AsmTrampoline["⚙️ Inline ASM (syscall; ret gadget)"]
                    ObfHash --> ResolveSSN --> AsmTrampoline
                end

                DecryptBeacon["🔓 Déchiffrement beacon.dll (XOR)"]
                FindExplorer["🔎 Trouver PID explorer.exe\n(NtQuerySystemInformation)"]
                RemoteMapping["🗺️ Remote Manual Mapping\n(Injection sans LoadLibrary)"]
                
                subgraph RemoteMappingSteps ["Étapes Mapping Distant"]
                    NtOpen["NtOpenProcess"]
                    NtAlloc["NtAllocateVirtualMemory (Remote)"]
                    MapSections["Écriture Headers/Sections (NtWrite)"]
                    PatchRelocs["Patch Relocations (Delta Base)"]
                    ResolveIATLocal["Résolution IAT (Locale -> Distante)"]
                    NtProtect["NtProtectVirtualMemory (RX/RW)"]
                    NtOpen --> NtAlloc --> MapSections --> PatchRelocs --> ResolveIATLocal --> NtProtect
                end

                InjectThread["💉 Injection Trampoline & Thread\n(NtCreateThreadEx -> DllMain)"]

                StartRust --> DecryptBeacon
                DecryptBeacon --> FindExplorer --> IndirectSyscalls
                IndirectSyscalls --> RemoteMapping
                RemoteMapping --> RemoteMappingSteps
                RemoteMappingSteps --> InjectThread
            end
            class Stage2_RustLoader rustLoader
        end


        %% --- Transition Link ---
        InjectThread ===>|"Création de Thread Distant"| DllMainTrigger

        %% --- Stage 3: Final Payload in Target Process ---
        subgraph TargetProc ["🎯 Processus Cible : explorer.exe (RAM)"]
            direction TB
            
            subgraph InjectedBeacon ["✨ Injected beacon.dll (Rust)"]
                direction TB
                DllMainTrigger["▶️ DllMain (PROCESS_ATTACH)"]
                SpawnWorker["🧵 Spawn Worker Thread"]

                subgraph Evasion ["🛡️ Evasion & Bypass"]
                    SetupVEH["🛠️ Setup VEH (Vectored Exception Handler)"]
                    SetHWBP["🎯 Set Hardware Breakpoints (Dr0-Dr3)\n(sur AmsiScanBuffer / NtTraceControl)"]
                    CatchException["⚡ Interception EXCEPTION_SINGLE_STEP"]
                    PatchRegs["🔧 Patch Registres (RAX=0 / RIP=RET)"]
                    SetupVEH --> SetHWBP --> CatchException --> PatchRegs
                end

                subgraph C2Loop ["🔄 Boucle C2 Discord"]
                    Polling["⏲️ Polling GET /messages (5s)"]
                    CmdParse["📥 Parsing Commande"]
                    
                    subgraph CommandEngine ["⚙️ Moteur de Commandes"]
                        CmdShell[">_ shell (PowerShell)"]
                        CmdScr["📷 scr (Screenshot Multi-mon)"]
                        CmdLoadDll["🧩 !loaddll (PE Loader interne)"]
                        CmdUac["🛡️ !uacbypass (CMSTPLUA)"]
                    end
                    
                    CmdExec["⚡ Exécution & Capture Output"]
                    ResultPost["📤 POST Résultat (JSON/Attachments)"]

                    Polling --> CmdParse --> CommandEngine --> CmdExec --> ResultPost
                    ResultPost -.->|"Attente prochain cycle"| Polling
                end

                DllMainTrigger --> SpawnWorker --> Evasion
                SpawnWorker --> C2Loop
            end
            class InjectedBeacon targetProcess
        end
    end

    %% --- Connections to C2 ---
    Polling <-->|"Trafic HTTPS légitime"| DiscordAPI
    ResultPost -->|"Upload résultats"| DiscordAPI

    %% --- Initial Flow ---
    PackedBinary -.->|"Double-clic / Exécution"| StartC