

fn main() {
    cc::Build::new()
        .files([
            //"libxmp-4.5.0/include/xmp.h",
            //"libxmp-4.5.0/win32/unistd.h",
            
            "libxmp-4.5.0/src/virtual.c",
            "libxmp-4.5.0/src/format.c",
            "libxmp-4.5.0/src/period.c",
            "libxmp-4.5.0/src/player.c",
            "libxmp-4.5.0/src/read_event.c",
            "libxmp-4.5.0/src/dataio.c",
            "libxmp-4.5.0/src/misc.c",
            //"libxmp-4.5.0/src/mkstemp.c",
            "libxmp-4.5.0/src/md5.c",
            "libxmp-4.5.0/src/lfo.c",
            "libxmp-4.5.0/src/scan.c",
            "libxmp-4.5.0/src/control.c",
            "libxmp-4.5.0/src/med_extras.c",
            "libxmp-4.5.0/src/filter.c",
            "libxmp-4.5.0/src/effects.c",
            "libxmp-4.5.0/src/mixer.c",
            "libxmp-4.5.0/src/mix_all.c",
            "libxmp-4.5.0/src/load_helpers.c",
            "libxmp-4.5.0/src/load.c",
            "libxmp-4.5.0/src/hio.c",
            //"libxmp-4.5.0/src/hmn_extras.c",
            //"libxmp-4.5.0/src/far_extras.c",
            "libxmp-4.5.0/src/extras.c",
            "libxmp-4.5.0/src/smix.c",
            //"libxmp-4.5.0/src/filetype.c",
            "libxmp-4.5.0/src/memio.c",
            "libxmp-4.5.0/src/tempfile.c",
            "libxmp-4.5.0/src/mix_paula.c",
            "libxmp-4.5.0/src/win32.c",
            "libxmp-4.5.0/src/loaders/common.c",
            "libxmp-4.5.0/src/loaders/iff.c",
            "libxmp-4.5.0/src/loaders/itsex.c",
            "libxmp-4.5.0/src/loaders/asif.c",
            "libxmp-4.5.0/src/loaders/lzw.c",
            "libxmp-4.5.0/src/loaders/voltable.c",
            "libxmp-4.5.0/src/loaders/sample.c",
            "libxmp-4.5.0/src/loaders/xm_load.c",
            "libxmp-4.5.0/src/loaders/mod_load.c",
            "libxmp-4.5.0/src/loaders/s3m_load.c",
            "libxmp-4.5.0/src/loaders/stm_load.c",
            "libxmp-4.5.0/src/loaders/669_load.c",
            "libxmp-4.5.0/src/loaders/far_load.c",
            "libxmp-4.5.0/src/loaders/mtm_load.c",
            "libxmp-4.5.0/src/loaders/ptm_load.c",
            "libxmp-4.5.0/src/loaders/okt_load.c",
            "libxmp-4.5.0/src/loaders/ult_load.c",
            "libxmp-4.5.0/src/loaders/mdl_load.c",
            "libxmp-4.5.0/src/loaders/it_load.c",
            "libxmp-4.5.0/src/loaders/stx_load.c",
            "libxmp-4.5.0/src/loaders/pt3_load.c",
            "libxmp-4.5.0/src/loaders/sfx_load.c",
            "libxmp-4.5.0/src/loaders/flt_load.c",
            "libxmp-4.5.0/src/loaders/st_load.c",
            "libxmp-4.5.0/src/loaders/emod_load.c",
            "libxmp-4.5.0/src/loaders/imf_load.c",
            "libxmp-4.5.0/src/loaders/digi_load.c",
            "libxmp-4.5.0/src/loaders/fnk_load.c",
            "libxmp-4.5.0/src/loaders/ice_load.c",
            "libxmp-4.5.0/src/loaders/liq_load.c",
            "libxmp-4.5.0/src/loaders/ims_load.c",
            "libxmp-4.5.0/src/loaders/masi_load.c",
            "libxmp-4.5.0/src/loaders/amf_load.c",
            "libxmp-4.5.0/src/loaders/psm_load.c",
            "libxmp-4.5.0/src/loaders/stim_load.c",
            "libxmp-4.5.0/src/loaders/mmd_common.c",
            "libxmp-4.5.0/src/loaders/mmd1_load.c",
            "libxmp-4.5.0/src/loaders/mmd3_load.c",
            "libxmp-4.5.0/src/loaders/rtm_load.c",
            "libxmp-4.5.0/src/loaders/dt_load.c",
            "libxmp-4.5.0/src/loaders/no_load.c",
            "libxmp-4.5.0/src/loaders/arch_load.c",
            "libxmp-4.5.0/src/loaders/sym_load.c",
            "libxmp-4.5.0/src/loaders/med2_load.c",
            "libxmp-4.5.0/src/loaders/med3_load.c",
            "libxmp-4.5.0/src/loaders/med4_load.c",
            "libxmp-4.5.0/src/loaders/dbm_load.c",
            "libxmp-4.5.0/src/loaders/umx_load.c",
            "libxmp-4.5.0/src/loaders/gdm_load.c",
            "libxmp-4.5.0/src/loaders/pw_load.c",
            "libxmp-4.5.0/src/loaders/gal5_load.c",
            "libxmp-4.5.0/src/loaders/gal4_load.c",
            "libxmp-4.5.0/src/loaders/mfp_load.c",
            "libxmp-4.5.0/src/loaders/asylum_load.c",
            "libxmp-4.5.0/src/loaders/hmn_load.c",
            "libxmp-4.5.0/src/loaders/mgt_load.c",
            "libxmp-4.5.0/src/loaders/chip_load.c",
            "libxmp-4.5.0/src/loaders/abk_load.c",
            "libxmp-4.5.0/src/loaders/coco_load.c",
        ])
        .define("_REENTRANT", None)
        .define("LIBXMP_NO_PROWIZARD", None)
        .define("LIBXMP_NO_DEPACKERS", None)
        //.define("LIBXMP_CORE_PLAYER", Some("1"))
        .include("libxmp-4.5.0/include")
        .include("libxmp-4.5.0/src")
        .include("libxmp-4.5.0/src/win32")
        //.include("libxmp-4.5.0/src/os2")
        .flag_if_supported("-flto=thin")
        .warnings(false)
        .compile("libxmp");
}