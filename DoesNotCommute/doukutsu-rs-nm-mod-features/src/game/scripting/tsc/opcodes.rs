use num_derive::FromPrimitive;

/// Engine's text script VM operation codes.
#[derive(EnumString, Debug, FromPrimitive, PartialEq, Copy, Clone)]
pub enum TSCOpCode {
    // ---- Internal opcodes (used by bytecode, no TSC representation)
    /// internal: no operation
    _NOP = 0,
    /// internal: unimplemented
    _UNI,
    /// internal: string marker
    _STR,
    /// internal: implicit END marker
    _END,

    // ---- Vanilla opcodes ----
    /// <BOAxxxx, Starts boss animation
    BOA,
    /// <BSLxxxx, Starts boss fight
    BSL,

    /// <FOBxxxx:yyyy, Focuses on boss part xxxx and sets speed to yyyy ticks
    FOB,
    /// <FOMxxxx, Focuses on player and sets speed to xxxx
    FOM,
    /// <FONxxxx:yyyy, Focuses on NPC tagged with event xxxx and sets speed to yyyy
    FON,
    /// <FLA, Flashes screen
    FLA,
    /// <QUAxxxx, Starts quake for xxxx ticks
    QUA,

    /// <UNIxxxx, Sets player movement mode (0 = normal, 1 = main artery)
    UNI,
    /// <HMC, Hides the player
    HMC,
    /// <SMC, Shows the player
    SMC,
    /// <MM0, Halts horizontal movement
    MM0,
    /// <MOVxxxx:yyyy, Moves the player to tile (xxxx,yyyy)
    MOV,
    /// <MYBxxxx, Bumps the player from direction xxxx
    MYB,
    /// <MYDxxxx, Makes the player face direction xxxx
    MYD,
    /// <TRAxxxx:yyyy:zzzz:wwww, Travels to map xxxx, starts event yyyy, places the player at tile (zzzz,wwww)
    TRA,

    /// <END, Ends the current event
    END,
    /// <FRE, Starts world ticking and unlocks player controls.
    FRE,
    /// <FAIxxxx, Fades in with direction xxxx
    FAI,
    /// <FAOxxxx, Fades out with direction xxxx
    FAO,
    /// <WAIxxxx, Waits for xxxx frames
    WAI,
    /// <WASs, Waits until the player is standing
    WAS,
    /// <KEY, Locks out the player controls.
    KEY,
    /// <PRI, Stops world ticking and locks out player controls.
    PRI,
    /// <NOD, Waits for input
    NOD,
    /// <CAT, Instantly displays the text, works for entire event
    CAT,
    /// <SAT, Same as <CAT
    SAT,
    /// <TUR, Instantly displays the text, works until <MSG/2/3 or <END
    TUR,
    /// <CLO, Closes the text box
    CLO,
    /// <CLR, Clears the text box
    CLR,
    /// <FACxxxx, Shows the face xxxx in text box, 0 to hide,
    /// CS+ Switch extensions:
    /// - add 0100 to display talking animation (requires faceanm.dat)
    /// - add 1000 to the number to display the face in opposite direction.
    /// Note that those extensions are enabled on every mod by default.
    FAC,
    /// <GITxxxx, Shows the item xxxx above text box, 0 to hide
    GIT,
    /// <MS2, Displays text on top of the screen without background.
    MS2,
    /// <MS3, Displays text on top of the screen with background.
    MS3,
    /// <MSG, Displays text on bottom of the screen with background.
    MSG,
    /// <NUMxxxx, Displays a value from AM+, buggy in vanilla.
    NUM,

    /// <ANPxxxx:yyyy:zzzz, Changes the animation state of NPC tagged with
    /// event xxxx to yyyy and set the direction to zzzz
    ANP,
    /// <CNPxxxx:yyyy:zzzz, Changes the NPC tagged with event xxxx to type yyyy
    /// and makes it face direction zzzz
    CNP,
    /// <INPxxxx:yyyy:zzzz, Same as <CNP, but also sets NPC flag event_when_touched (0x100)
    INP,
    /// <MNPxxxx:yyyy:zzzz:wwww, Moves NPC tagged with event xxxx to tile position (xxxx,yyyy)
    /// and makes it face direction zzzz
    MNP,
    /// <DNAxxxx, Deletes all NPCs of type xxxx
    DNA,
    /// <DNPxxxx, Deletes all NPCs of type xxxx
    DNP,
    SNP,

    /// <FL-xxxx, Sets the flag xxxx to false
    #[strum(serialize = "FL-")]
    FLm,
    /// <FL+xxxx, Sets the flag xxxx to true
    #[strum(serialize = "FL+")]
    FLp,
    /// <MP-xxxx, Sets the map xxxx to true
    #[strum(serialize = "MP+")]
    MPp,
    /// <SK-xxxx, Sets the skip flag xxx to false
    #[strum(serialize = "SK-")]
    SKm,
    /// <SK+xxxx, Sets the skip flag xxx to true
    #[strum(serialize = "SK+")]
    SKp,

    /// <EQ+xxxx, Sets specified bits in equip bitfield
    #[strum(serialize = "EQ+")]
    EQp,
    /// <EQ-xxxx, Unsets specified bits in equip bitfield
    #[strum(serialize = "EQ-")]
    EQm,
    /// <ML+xxxx, Adds xxxx to maximum health.
    #[strum(serialize = "ML+")]
    MLp,
    /// <IT+xxxx, Adds item xxxx to players inventory.
    #[strum(serialize = "IT+")]
    ITp,
    /// <IT-xxxx, Removes item xxxx to players inventory.
    #[strum(serialize = "IT-")]
    ITm,
    /// <AM+xxxx:yyyy, Adds weapon xxxx with yyyy ammo (0 = infinite) to players inventory.
    #[strum(serialize = "AM+")]
    AMp,
    /// <AM-xxxx, Removes weapon xxxx from players inventory.
    #[strum(serialize = "AM-")]
    AMm,
    /// <TAMxxxx:yyyy:zzzz, Trades weapon xxxx for weapon yyyy with zzzz ammo
    TAM,

    /// <UNJxxxx:yyyy, Jumps to event yyyy if movement mode is equal to xxxx
    UNJ,
    /// <NCJxxxx:yyyy, Jumps to event yyyy if NPC of type xxxx is alive
    NCJ,
    /// <ECJxxxx:yyyy, Jumps to event yyyy if NPC tagged with event xxxx is alive
    ECJ,
    /// <FLJxxxx:yyyy, Jumps to event yyyy if flag xxxx is set
    FLJ,
    /// <FLJxxxx:yyyy, Jumps to event yyyy if player has item xxxx
    ITJ,
    /// <MPJxxxx, Jumps to event xxxx if map flag for current stage is set
    MPJ,
    /// <YNJxxxx, Jumps to event xxxx if prompt response is No, otherwise continues event execution
    YNJ,
    /// <MPJxxxx, Jumps to event xxxx if skip flag for is set
    SKJ,
    /// <EVExxxx, Jumps to event xxxx
    EVE,
    /// <AMJyyyy, Jumps to event xxxx player has weapon yyyy
    AMJ,

    /// <MLP, Displays the map of current stage
    MLP,
    /// <MLP, Displays the name of current stage
    MNA,
    /// <CMPxxxx:yyyy:zzzz, Sets the tile at (xxxx,yyyy) to type zzzz
    CMP,
    /// <SMPxxxx:yyyy, Subtracts 1 from tile type at (xxxx,yyyy)
    SMP,

    /// <CRE, Shows credits
    CRE,
    /// <XX1xxxx, Shows falling island
    XX1,
    /// <CIL, Hides credits illustration
    CIL,
    /// <SILxxxx, Shows credits illustration xxxx
    SIL,
    /// <ESC, Exits to title screen
    ESC,
    /// <INI, Exits to "Studio Pixel presents" screen
    INI,
    /// <LDP, Loads a saved game
    LDP,
    /// <PS+xxxx:yyyy, Sets teleporter slot xxxx to event number yyyy
    #[strum(serialize = "PS+")]
    PSp,
    /// <SLP, Shows the teleporter menu
    SLP,
    /// <ZAM, Resets the experience and level of all weapons
    ZAM,

    /// <AE+, Refills ammunition
    #[strum(serialize = "AE+")]
    AEp,
    /// <LI+xxxx, Recovers xxxx health
    #[strum(serialize = "LI+")]
    LIp,

    /// <SVP, Saves the current game
    SVP,
    /// <STC, Saves the state of Nikumaru counter
    STC,

    /// <SOUxxxx, Plays sound effect xxxx
    SOU,
    /// <CMUxxxx, Changes BGM to xxxx
    CMU,
    /// <FMU, Fades the BGM
    FMU,
    /// <RMU, Restores the music state of BGM played before current one
    RMU,
    /// <CPS, Stops the propeller sound
    CPS,
    /// <SPS, Starts the propeller sound
    SPS,
    /// <CSS, Stops the stream sound
    CSS,
    /// <SSSxxxx, Starts the stream sound at volume xxxx
    SSS,

    // ---- Cave Story+ specific opcodes ----
    /// <ACHxxxx, triggers a Steam achievement. No-op in EGS/Humble Bundle version.
    ACH,

    // ---- Cave Story+ (Switch) specific opcodes ----
    /// <HM2, HMC only for executor player.
    HM2,
    /// <2MVxxxx, Put another player near the player who executed the event.
    /// 0000 - puts player on left side of executor player
    /// 0001 - puts player on right side of executor player
    /// 0002-0010 - unused
    /// 0011.. - the first 3 digits are distance in pixels, the last digit is a flag
    ///        - if it's 1 put the player on right side of the player, otherwise put it on left
    #[strum(serialize = "2MV")]
    S2MV,
    /// <2PJ, jump to event if in multiplayer mode.
    #[strum(serialize = "2PJ")]
    S2PJ,
    /// <INJxxxx:yyyy:zzzz, Jumps to event zzzz if amount of item xxxx equals yyyy
    INJ,
    /// <I+Nxxxx:yyyy, Adds item xxxx with maximum amount of yyyy
    #[strum(serialize = "I+N")]
    IpN,
    /// <FF-xxxx:yyyy, Sets first flag in range xxxx-yyyy to false
    #[strum(serialize = "FF-")]
    FFm,
    /// <PSHxxxx, Pushes text script state to stack and starts event xxxx
    PSH,
    /// <POP, Restores text script state from stack and resumes previous event, otherwise works like half-baked `<END`
    POP,
    /// <KEY related to player 2?
    KE2,
    /// <FRE related to player 2?
    FR2,
    // ---- Custom opcodes, for use by modders ----

    /// <CMLwwww:xxxx:yyyy:zzzz, Sets the tile at (xxxx,yyyy) to type zzzz, on layer wwww [0/back, 1/mid, 2/fore, 3/far fore]
    CML,
    /// <SMLwwww:xxxx:yyyy, Subtracts 1 from tile type at (xxxx,yyyy) on layer wwww [0/back, 1/mid, 2/fore, 3/far fore]
    SML,


    /// <BKGname_of_config$, Loads the BKG config file with name_of_config
    BKG,
    /// <BKDwwww, Disable the backgroubd layer wwww (out-of-range layers will be set to the last layer)
    BKD,
    /// <BKEwwww, Enable background layer (simmilar to BKD)
    BKE,
    /// <BKPwwww:xxxx:yyyy, Set BKG parameter xxxx for layer wwww to value yyyy (TODO: negatives and floating points)
    BKP,
    /// <BKR, Restores background to default parameters for the map, simmilar to a TRA command
    BKR,

    /// <SVMname_of_profile$, saves profile to location: name_of_profile
    SVM,
    /// <LDMname_of_profile$, loads profile from location: name_of_profile
    LDM,

    /// <MIMwwww:name_of_skin$, (multi-MIMiga-mask) Load in a player skin to player wwww with name_of_skin (note: starting directory is ./data/Skins)
    MIM,

    /// <TCLwwww:xxxx:yyyy, TimerControL, `<TCL[unit FPS: 0050, 0060, otherwise current]:[start time (seconds)]:[event to run when timer is 0]` (starting and stopping is part of the equp list: 0512)
    TCL,
    /// <ADTwwww:xxxx:yyyy, ADjustTime `<ADT[unit FPS: 0050, 0060, otherwise current]:[0000+/1-]:[time (seconds)]`, adds or subs time to the n. timer
    ADT,
    /// <SLTwwww:name_of_file$, SaveLoadTime, Saves the current time to the file specified by the name `<SLT[0000Save/1Load]:name_of_file`, starts in the user save directory
    SLT,

    /// <NIMwwww:name_of_skin$, (NPC mIMiga-mask) initialize NPC wwww's skin to use a cusom player skin (like <MIM), leave empty to un-initialize it
    NIM,

    /// <ALCwwww:name_of_logfile$, Action Logger Control, controls the logging of PC action `<ALC[0Stop/1Start]`
    ALC,
    /// <ARLwwww:name_of_logfile$, Action Reader Load, loads the action file into NPC wwww, loading "" removes the file from memory (Control is done through NPC ANPs)
    ARL,

    /// <LISwwww:xxxx, LIfe Set, sets the player's max (wwww) and current (xxxx) HP
    LIS,

    /// <AMLwwww:xxxx:yyyy, ArMs Level, sets the starting level of a player's weapon `<AML[Weapon ID]:[Level]:[EXP]`
    AML,

    /// <UFJwwww:name_of_file$, User File Jump, jumps TSC if the file in the user directory does not exist
    UFJ,

    /// <DFJwwww:name_of_file$, Data FIle Jump, jumps TSC if the file in the data directory does not exist
    DFJ,

    /// <FNJxxxx:yyyy, Jumps to event yyyy if flag xxxx is NOT set
    FNJ,

    /// <CFGwwww:xxxx:yyyy:zzzz, ConfiG npc, gives all wwww action_num xxxx, tsc_direction yyyy, and action_counter zzzz, then runs it for 1 tick immediately
    CFG,

    /// <MMLwwww:xxxx:yyyy:zzzz, Make Map Layer, Sets the tile at (xxxx,yyyy) to type zzzz, on layer wwww [0/back, 1/mid, 2/fore, 3/far fore] (no smoke)
    MML,

    /// <RETxxxx:yyyy:zzzz:wwww, RETurn to title screen, Travels to map xxxx, starts event yyyy, places the player at tile (zzzz,wwww)
    RET,

    /// <TIJwwww:name_of_file$, Jump to wwww if the current time is more than the time in the file
    TIJ,

    /// <REPwwww, Set the "replay mode" flag (like KEY and FRE), <REP[0-false/1-true] (may be redundant and removed later)
    REP,


    // <CMFwwww:name_of_file$ Cue Music File, loads and starts music from `./data` (subdirectories can be included)
    CMF,

    // <UCPname_of_src$:name_of_dst$ User File Copy: Copies file contents from src to dst in the user directory
    UFC,

    // <LSUwwww:name_of_file$, Load SUrface: Loads the bitmap with name to surface ID wwww (because of the way d-rs handles spritesheets, only the values represented in StageTexturePaths can be changed)
    LSU


}

#[derive(FromPrimitive, PartialEq, Copy, Clone)]
pub enum CreditOpCode {
    /// Internal, no operation
    _NOP = 0,
    /// `/`
    ///
    /// Arguments: `()`
    StopCredits,

    /// `[{text: string}]{cast_tile: number}`
    ///
    /// Arguments: `(cast_tile: varint, text_len: varint, text: [varint; text_len])`
    PushLine,

    /// `-{ticks: number}`
    ///
    /// Arguments: `(ticks: varint)`
    Wait,

    /// `+{offset: number}`
    ///
    /// Arguments: `(offset: varint)`
    ChangeXOffset,

    /// `!{music_id: number}`
    ///
    /// Arguments: `(music_id: varint)`
    ChangeMusic,

    /// `~`
    ///
    /// Arguments: `()`
    FadeMusic,

    /// `j{label: number}`
    ///
    /// Arguments: `(label: varint)`
    JumpLabel,

    /// `f{flag: number}:{label: number}`
    ///
    /// Arguments: `(flag: varint, label: varint)`
    JumpFlag,

    // ---- Cave Story+ (Switch) specific opcodes ----
    /// `p2:{label: number}`
    ///
    /// Arguments: `(label: varint)`
    JumpPlayer2,
}
