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
    /// <POP, Restores text script state from stack and resumes previous event.
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

    /// <MS4, Displays text on the bottom of the screen without background. (like MS2, but where MSG happens)
    MS4,


    /// <L3Dwwww:yyyy:path_to_file$, loads a GLTF file from a path in the "./data" directory to storage key wwww, if yyyy is 1, lights are imported from this model, invalid path unloads the model
    L3D,
    /// <LSBwwww:path_to_file$, loads an image from the "./data" directory to be used as a skybox (equirectangular), if wwww is one, then the game uses this texture for the ambient light, invalid path unloads the skybox
    LSB,
    /// <LAIpath_to_file$, loads a equirectangular texture to be used with the ambient light. if string is invalid, the current texture is unloaded
    LAI,    
    /// <SAIwwww, sets the intensity of the ambient light / 1000, so 0500 is 0.5.
    SAI,
    /// <SACwwww:xxxx:yyyy, sets color of the ambient light, RGB from 0-255. Values outside this are truncated
    SAC,

    ///Animation 3d Select, choose what animation this model should play back. Because of the backend, only one animation can be played at a time
    /// <3AS[key]:[name_of_animation$]
    #[strum(serialize = "3AS")]
    S3AS,

    /// 3d Animation Time, set the time in seconds along this animation's timeline (1000ms = 1s)
    ///<3AT[key]:[time milliseconds]
    #[strum(serialize = "3AT")]
    S3AT,

    /// 3d Animation Go, start the animation playing. Note: time increment is relative to ingame tick speed, by default, they happen by 1/50 seconds each frame.
    /// this means they will play faster with 60 TPS than with 50 TPS
    /// <3AG[key]
    #[strum(serialize = "3AG")]
    S3AG,

    /// 3d Animation Pause, stop the animation playing
    /// <3AP[key]
    #[strum(serialize = "3AP")]
    S3AP,
    
    /// 3d Animation Range, starts the animation and plays it until it is >= the stop time, then it pauses it and snaps it to this time
    /// <3AR[key]:[stop time in millisconds]
    #[strum(serialize = "3AR")]
    S3AR,

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