use crate::dcl::ProcEngine;
use crate::dcl::*;
use crate::p08::p08_class_val;
use crate::p08::ProfType;
use crate::utl::mon_kwh_2_kw;
use crate::utl::trf_kva_2_kw;
use crate::utl::*;
//use iterstats::Iterstats;
use regex::Regex;
use sglib04::geo4::PowerProdType;
use std::collections::HashMap;
use std::error::Error;

//pub const EV_CHG_PROF_KW: f32 = 0.42;
//pub const EV_CHG_PROF_KW: f32 = 1.0;
pub const EV_CHG_PROF_KW: f32 = 1.2;

pub const EV_CHG_POW_KW: f32 = 7.0;
pub const EV_DAY_CHG_HOUR: f32 = 2.0;
pub const EV_CLAIM_RATE: f32 = 1.0;

// EV truck
//pub const ET_CHG_POW_KW: f32 = 300f32;
pub const ET_CHG_POW_KW: f32 = 200f32;
pub const ET_DAY_CHG_HOUR: f32 = 4.0;
pub const ET_CLAIM_RATE: f32 = 1.0;

// EV bike
//pub const EB_CHG_POW_KW: f32 = 0.2f32;
pub const EB_CHG_POW_KW: f32 = 0.1f32;
pub const EB_DAY_CHG_HOUR: f32 = 3.0;
pub const EB_CLAIM_RATE: f32 = 1.0;

use crate::cst2::cst_bes_imp;
use crate::cst2::cst_bes_ins;
use crate::cst2::cst_bes_op;
use crate::cst2::cst_comm_imp;
use crate::cst2::cst_comm_ins;
use crate::cst2::cst_comm_op;
use crate::cst2::cst_m1p_imp;
use crate::cst2::cst_m1p_ins;
use crate::cst2::cst_m1p_op;
use crate::cst2::cst_m3p_imp;
use crate::cst2::cst_m3p_ins;
use crate::cst2::cst_m3p_op;
use crate::cst2::cst_plfm_imp;
use crate::cst2::cst_plfm_ins;
use crate::cst2::cst_plfm_op;
use crate::cst2::cst_tr_imp;
use crate::cst2::cst_tr_ins;
use crate::cst2::cst_tr_op;
use crate::cst2::eir_cust_etruck_save;
use crate::cst2::eir_cust_ev_save;
use crate::cst2::eir_cust_loss_save;
use crate::cst2::eir_cust_mv_rev;
use crate::cst2::eir_cust_save;
use crate::cst2::eir_cust_solar_roof;
use crate::cst2::eir_en_rev_save;
use crate::cst2::eir_ghg_save;
use crate::stg3::NON_TECH_LOSS_RATIO;
use crate::stg3::NOTEC_LOSS_CLAIM_RATE;
use crate::stg3::SAVE_LOSS_UNIT_PRICE;
use crate::stg3::TRANS_REPL_CLAIM_RATE;
use crate::stg3::TRANS_REPL_UNIT_PRICE;
use crate::stg3::TRANS_REPL_WITHIN_YEAR;
use crate::stg3::UNBAL_CALC_FACTOR;
use crate::stg3::UNBAL_HOUR_PER_DAY;
use crate::stg3::UNBAL_LOSS_CLAIM_RATE;
use crate::stg3::UNBAL_REPL_CLAIM_RATE;
use std::sync::Arc;
use std::sync::Mutex;

pub const BESS_EVPOW_MWH_MULT: f32 = 1.0;
use crate::cst2::cst_reinvest;
use crate::stg3::REINVEST_RATE;
use sglib04::web1::ENERGY_GRW_RATE;

/// read 000_pea.bin
/// read SSS.bin
pub fn stage_02() -> Result<(), Box<dyn Error>> {
    println!("===== STAGE 2 =====");
    let buf = std::fs::read(format!("{DNM}/000_pea.bin")).unwrap();
    let (pea, _): (Pea, usize) =
        bincode::decode_from_slice(&buf[..], bincode::config::standard()).unwrap();
    println!("pea ar:{}", pea.aream.len());
    let mut aids: Vec<_> = pea.aream.keys().collect();
    aids.sort();
    //let mut tras_mx1 = PeaAssVar::default();
    //let mut tras_mx2 = PeaAssVar::default();
    let mut tras_mx1 = PeaAssVar::from(0u64);
    let mut assrw1 = Vec::<PeaAssVar>::new();
    stage_02_1(&aids, &pea, &mut assrw1, &mut tras_mx1)?;
    println!(
        "====================================== RAW1 len: {}",
        assrw1.len()
    );
    stage_02_b(assrw1, &tras_mx1)?;
    /*
    let mut assrw2 = Vec::<PeaAssVar>::new();
    stage_02_2(
        &aids,
        &pea,
        &mut assrw1,
        &mut assrw2,
        &tras_mx1,
        &mut tras_mx2,
        &mut tras_sm2,
    )?;
    println!(
        "====================================== RAW2 len: {}",
        assrw2.len()
    );
    stage_02_3(&aids, &pea, DNM, &tras_mx2, &tras_sm2)?;
    stage_02_4(&aids, &pea, DNM)?;
    let maxs = vec![tras_mx1, tras_mx2, tras_sm2];
    let bin: Vec<u8> = bincode::encode_to_vec(&maxs, bincode::config::standard()).unwrap();
    std::fs::write(format!("{DNM}/pea-mx.bin"), bin).unwrap();
    */

    Ok(())
}

/*
pub fn stage_02_a() -> Result<(), Box<dyn Error>> {
    println!("===== STAGE 2A =====");
    let buf = std::fs::read(format!("{DNM}/000_pea.bin")).unwrap();
    let (pea, _): (Pea, usize) =
        bincode::decode_from_slice(&buf[..], bincode::config::standard()).unwrap();
    let mut aids: Vec<_> = pea.aream.keys().collect();
    aids.sort();
    stage_02_4(&aids, &pea, DNM)?;
    Ok(())
}
*/

pub fn stage_02_1(
    aids: &Vec<&String>,
    pea: &Pea,
    assv: &mut Vec<PeaAssVar>,
    tras_mx1: &mut PeaAssVar,
) -> Result<(), Box<dyn Error>> {
    let e0 = ProcEngine::prep5();
    let keys: Vec<_> = e0.lp24.keys().collect();
    let re = Regex::new(r"([A-Z]{3})-([0-9]{2})[VW].*").unwrap();
    let mut fd2fd = HashMap::<String, String>::new();
    for k in keys {
        for cap in re.captures_iter(k) {
            let a = &cap[1].to_string();
            let b = &cap[2].to_string();
            let fd = format!("{a}{b}");
            if let Some(o) = fd2fd.get(&fd) {
                println!("DUP {o} => fd:{fd} k:{k}");
            } else {
                fd2fd.insert(fd, k.to_string());
            }
        }
    }
    let a_fd2fd = Arc::new(Mutex::new(fd2fd));
    let a_e0 = Arc::new(Mutex::new(e0));
    let a_tras_mx1 = Arc::new(Mutex::new(tras_mx1));
    let a_assv = Arc::new(Mutex::new(assv));

    std::thread::scope(|s| {
        for id in aids {
            let aid = id.to_string();

            let c_e0 = a_e0.clone();
            let c_fd2fd = a_fd2fd.clone();
            let c_tras_mx1 = a_tras_mx1.clone();
            let c_assv = a_assv.clone();

            let id = id.to_string();
            let aream = pea.aream.clone();
            let _handle = s.spawn(move || {
                let Some(ar) = aream.get(&aid) else {
                    return;
                };
                println!("ar:{}", ar.arid);
                let eg = ProcEngine::prep3(&id);
                //--- amphor initialization
                let mut am_dn = HashMap::<String, (f32, f32)>::new();
                for dn in &eg.amps {
                    let key = dn.key.to_string();
                    if let Some((_po, aa)) = am_dn.get_mut(&key) {
                        *aa += dn.area;
                    } else {
                        am_dn.insert(key, (dn.popu, dn.area));
                    }
                }
                //--- municipality initialization
                let mut mu_dn = HashMap::<String, (f32, f32)>::new();
                for dn in &eg.muni {
                    let key = dn.key.to_string();
                    if let Some((_po, aa)) = mu_dn.get_mut(&key) {
                        *aa += dn.area;
                    } else {
                        mu_dn.insert(key, (dn.popu, dn.area));
                    }
                }
                let mut pids: Vec<_> = ar.provm.keys().collect();
                pids.sort();
                // province loop1
                for pid in pids {
                    let Some(prov) = ar.provm.get(pid) else {
                        continue;
                    };
                    let vp01 = prov.evpc;
                    let vp02 = prov.gppv;
                    let evlk = *EV_LIKELY.get(pid).unwrap_or(&0f32);
                    let selk = *SELE_LIKELY.get(pid).unwrap_or(&0f32);
                    //println!("  pv:{pid}");
                    let mut sids: Vec<_> = prov.subm.keys().collect();
                    sids.sort();
                    for sid in sids {
                        // check if the substation exists
                        let Some(_sb) = prov.subm.get(sid) else {
                            continue;
                        };
                        // read substation data from storage
                        let Ok(buf) = std::fs::read(format!("{DNM}/{sid}.bin")) else {
                            continue;
                        };
                        let (sub, _): (PeaSub, usize) =
                            bincode::decode_from_slice(&buf[..], bincode::config::standard())
                                .unwrap();
                        // Substation
                        //=====================================================
                        //=====================================================
                        let mut vs01 = 0f32;
                        let mut vs02 = 0f32;
                        let mut solar = 0f32;
                        if let Ok(e0) = c_e0.lock() {
                            if let Some(lp) = e0.lp24.get(sid) {
                                for v in lp.pos_rep.val.into_iter().flatten() {
                                    vs01 = vs01.max(v.unwrap_or(0f32));
                                }
                            } else if let Some(lp) = e0.lp23.get(sid) {
                                for v in lp.pos_rep.val.into_iter().flatten() {
                                    vs01 = vs01.max(v.unwrap_or(0f32));
                                }
                            };
                            if let Some(lp) = e0.lp24.get(sid) {
                                for v in lp.neg_rep.val.into_iter().flatten() {
                                    vs02 = vs02.max(v.unwrap_or(0f32));
                                }
                            } else if let Some(lp) = e0.lp23.get(sid) {
                                for v in lp.neg_rep.val.into_iter().flatten() {
                                    vs02 = vs02.max(v.unwrap_or(0f32));
                                }
                            }
                            if let Some(lp) = e0.lp24.get(sid) {
                                if let Some(lpv) = &lp.pos_rep.val {
                                    if let Ok(lpf) = p08_class_val(lpv) {
                                        if lpf.lp_type == ProfType::SolarPower
                                            || lpf.lp_type == ProfType::SolarNight
                                        {
                                            solar = -lpf.sol_en.unwrap_or(0f32);
                                        }
                                    }
                                }
                            }
                        }
                        //  VSPP, SPP, RE plan
                        let mut vs03 = 0f32;
                        for pi in &sub.vspps {
                            vs03 += eg.vsps[*pi].kw.unwrap_or(0f32);
                        }
                        let mut vs04 = 0f32;
                        for pi in &sub.spps {
                            vs04 += eg.spps[*pi].mw.unwrap_or(0f32);
                        }
                        let mut vs05 = 0f32;
                        for pi in &sub.repls {
                            if let PowerProdType::SPP = eg.repl[*pi].pptp {
                                vs05 += eg.repl[*pi].pwmw.unwrap_or(0f32);
                            }
                        }
                        let mut vs06 = 0f32;
                        for pi in &sub.repls {
                            if let PowerProdType::VSPP = eg.repl[*pi].pptp {
                                vs06 += eg.repl[*pi].pwmw.unwrap_or(0f32);
                            }
                        }
                        let vs07 = sub.mvxn as f32;

                        println!(" pv:{pid} sb:{sid} feed: {}", sub.feeders.len());
                        let mut fids: Vec<_> = sub.feedm.keys().collect();
                        fids.sort();
                        let mut s_tr_ass = Vec::<PeaAssVar>::new();
                        for fid in fids {
                            let Some(fd) = sub.feedm.get(fid) else {
                                continue;
                            };
                            ////////////////////////////////////////////////////////
                            ////////////////////////////////////////////////////////
                            // Feeder
                            //let k1 = format!("{fid}");
                            let k1 = fid.to_string();
                            let key = if let Ok(fd2fd) = c_fd2fd.lock() {
                                fd2fd.get(&k1).unwrap_or(&"-".to_string()).to_string()
                            } else {
                                "".to_string()
                            };
                            let mut grw = EN_AVG_GRW_RATE;
                            let (mut af01, mut af03, mut af02, mut af04) = (None, None, None, None);

                            if let Ok(e0) = c_e0.lock() {
                                if let (Some(lp1), Some(lp0)) =
                                    (e0.lp24.get(&key), e0.lp23.get(&key))
                                {
                                    let mut pwmx = 0f32;
                                    if let Some(reps) = lp1.pos_rep.val {
                                        for vv in reps.iter().flatten() {
                                            pwmx = pwmx.max(*vv);
                                        }
                                    };
                                    let mut pwmx0 = 0f32;
                                    if let Some(reps) = lp0.pos_rep.val {
                                        for vv in reps.iter().flatten() {
                                            pwmx0 = pwmx0.max(*vv);
                                        }
                                    };

                                    let grw2 = if pwmx0 > 0f32 {
                                        (pwmx - pwmx0) / pwmx * 100f32
                                    } else {
                                        0f32
                                    };
                                    if grw2 > grw && grw2 < EN_MAX_GRW_RATE {
                                        grw = grw2;
                                    }
                                }

                                if let Some(lp) = e0.lp24.get(&key) {
                                    if let Some(vv) = lp.pos_rep.val {
                                        for v in vv.iter().flatten() {
                                            if let Some(v0) = af01 {
                                                af01 = Some(v.max(v0))
                                            } else {
                                                af01 = Some(*v);
                                            }
                                            if let Some(v0) = af03 {
                                                af03 = Some(v.min(v0))
                                            } else {
                                                af03 = Some(*v);
                                            }
                                        }
                                    }
                                    if let Some(vv) = lp.neg_rep.val {
                                        for v in vv.iter().flatten() {
                                            if let Some(v0) = af02 {
                                                af02 = Some(v.max(v0))
                                            } else {
                                                af02 = Some(*v);
                                            }
                                            if let Some(v0) = af04 {
                                                af04 = Some(v.min(v0))
                                            } else {
                                                af04 = Some(*v);
                                            }
                                        }
                                    }
                                } else if let Some(lp) = e0.lp23.get(&key) {
                                    if let Some(vv) = lp.pos_rep.val {
                                        for v in vv.iter().flatten() {
                                            if let Some(v0) = af01 {
                                                af01 = Some(v.max(v0))
                                            } else {
                                                af01 = Some(*v);
                                            }
                                            if let Some(v0) = af03 {
                                                af03 = Some(v.min(v0))
                                            } else {
                                                af03 = Some(*v);
                                            }
                                        }
                                    }
                                    if let Some(vv) = lp.neg_rep.val {
                                        for v in vv.iter().flatten() {
                                            if let Some(v0) = af02 {
                                                af02 = Some(v.max(v0))
                                            } else {
                                                af02 = Some(*v);
                                            }
                                            if let Some(v0) = af04 {
                                                af04 = Some(v.min(v0))
                                            } else {
                                                af04 = Some(*v);
                                            }
                                        }
                                    }
                                };
                            }
                            let vf01 = af01.unwrap_or(0f32);
                            let mut vf03 = af03.unwrap_or(0f32);
                            vf03 = vf01 - vf03;
                            let vf02 = af02.unwrap_or(0f32);
                            let mut vf04 = af04.unwrap_or(0f32);
                            vf04 = vf02 - vf04;

                            let mut tids: Vec<_> = fd.tranm.keys().collect();
                            tids.sort();

                            // =========================
                            // loop on each transformer
                            for tid in tids {
                                let Some(trn) = fd.tranm.get(tid) else {
                                    continue;
                                };
                                let aojs = trn.aojs.clone();
                                let vt05 = trn.tr_kva.unwrap_or(10f32);
                                let vt05 = trf_kva_2_kw(vt05);
                                let mut vt06 = 1f32;
                                for zi in &trn.zons {
                                    match eg.zons[*zi].zncd.clone().expect("-").as_str() {
                                        "21" | "22" | "24" => {
                                            vt06 = vt06.max(5f32);
                                        }
                                        "11" | "12" | "13" | "14" => {
                                            vt06 = vt06.max(4f32);
                                        }
                                        "23" | "25" | "31" => {
                                            vt06 = vt06.max(3f32);
                                        }
                                        "41" | "42" => {
                                            vt06 = vt06.max(2f32);
                                        }
                                        _ => {}
                                    }
                                }
                                let (aoj, aojcd) = if aojs.is_empty() {
                                    ("-".to_string(), "-".to_string())
                                } else {
                                    let ai = aojs[0];
                                    let aoj =
                                        eg.aojs[ai].sht_name.clone().unwrap_or("-".to_string());
                                    let aojcd = eg.aojs[ai].code.clone().unwrap_or("-".to_string());
                                    (aoj, aojcd)
                                };

                                let mut vt01 = 0f32;
                                let mut vt02 = 0f32;
                                let mut vt10 = 0f32;
                                let mut nom1p = 0f32;
                                let mut nom3p = 0f32;
                                let mut allsel = 0f32;

                                let (mut se_a, mut se_b, mut se_c) = (0.0, 0.0, 0.0);
                                let (mut sl_a, mut sl_b, mut sl_c, mut sl_3) = (0.0, 0.0, 0.0, 0.0);
                                //
                                // =========================
                                // loop on each meter
                                for met in &trn.mets {
                                    ///////////////////////////////////////////////////
                                    ///////////////////////////////////////////////////
                                    // Meter
                                    if let MeterAccType::Small = met.met_type {
                                        if met.main.is_empty() && met.kwh18 > 600f32 {
                                            //if met.main.is_empty() && met.kwh18 > 200f32 {
                                            vt01 += 1f32;
                                            vt02 += met.kwh15;
                                        }
                                        allsel += met.kwh15;
                                    } else if let MeterAccType::Large = met.met_type {
                                        vt10 += met.kwh15;
                                        print!("_{}", met.kwh15);
                                        //allsel += met.kwh15;
                                    }
                                    if trn.own == "P" {
                                        match met.mt_phs.clone().unwrap_or(String::new()).as_str() {
                                            "A" => se_a += met.kwh15,
                                            "B" => se_b += met.kwh15,
                                            "C" => se_c += met.kwh15,
                                            _ => {}
                                        }
                                        match met.mt_phs.clone().unwrap_or(String::new()).as_str() {
                                            "A" => sl_a += met.kwh15,
                                            "B" => sl_b += met.kwh15,
                                            "C" => sl_c += met.kwh15,
                                            _ => sl_3 += met.kwh15,
                                        }
                                        match met.mt_phs.clone().unwrap_or(String::new()).as_str() {
                                            "A" | "B" | "C" => nom1p += 1.0,
                                            _ => nom3p += 1.0,
                                        }
                                    }
                                }
                                let vt11 = trn.mets.len() as f32;
                                let vt12 = 1f32;
                                sl_3 = mon_kwh_2_kw(sl_3);
                                sl_a = mon_kwh_2_kw(sl_a);
                                sl_b = mon_kwh_2_kw(sl_b);
                                sl_c = mon_kwh_2_kw(sl_c);
                                let v_phs_a = sl_3 + sl_a;
                                let v_phs_b = sl_3 + sl_b;
                                let v_phs_c = sl_3 + sl_c;
                                let v_all_p = sl_3 + sl_a + sl_b + sl_c;
                                let v_ph_av = (v_phs_a + v_phs_b + v_phs_c) / 3f32;
                                let v_ph_mx = v_phs_a.max(v_phs_b.max(v_phs_c));
                                let v_ph_rt = v_ph_mx / z2o(v_ph_av);
                                let v_al_kw = v_phs_a + v_phs_b + v_phs_c;
                                let v_loss = v_al_kw * TRF_LOSS_RATIO;
                                let v_unba = v_loss * TRF_UNBAL_K * v_ph_rt * v_ph_rt;
                                let v_unb_sat = v_ph_mx / z2o(vt05);
                                let v_unb_cnt = if v_unb_sat >= TRF_UNBAL_CNT_RATE {
                                    1f32
                                } else {
                                    0f32
                                };
                                let v_max_sat = v_all_p / z2o(vt05);
                                let v_max_cnt =
                                    if v_unb_cnt == 0f32 && v_max_sat >= TRF_UNBAL_CNT_RATE {
                                        1f32
                                    } else {
                                        0f32
                                    };
                                let mut vt08 = 0f32;
                                let se_p = se_a + se_b + se_c;
                                if se_a < se_p && se_b < se_p && se_c < se_p {
                                    let ab = (se_a - se_b).abs();
                                    let bc = (se_b - se_c).abs();
                                    let ca = (se_c - se_a).abs();
                                    vt08 = (ab + bc + ca) * 0.5;
                                }
                                let vt08 = mon_kwh_2_kw(vt08);
                                //let vt09 = trf_kva_2_kw(vt02);
                                let vt09 = mon_kwh_2_kw(vt02);

                                let mut vt03 = 0f32;
                                for vi in &trn.vols {
                                    for (pw, no) in &eg.vols[*vi].chgr {
                                        vt03 += (*pw * *no) as f32;
                                    }
                                }
                                let mut vt04 = 0f32;
                                for vi in &trn.vols {
                                    for (_yr, am) in &eg.vols[*vi].sell {
                                        vt04 += *am;
                                    }
                                }
                                let mut vt07 = 1f32;
                                for ai in &trn.amps {
                                    let am = &eg.amps[*ai].key;
                                    if let Some((p, a)) = am_dn.get(am) {
                                        let a = a / 1_000f32;
                                        let pd = p / a * 0.6f32;
                                        let v = match pd {
                                            0f32..30f32 => 1f32,
                                            30f32..60f32 => 2f32,
                                            60f32..150f32 => 3f32,
                                            150f32..500f32 => 4f32,
                                            _ => 5f32,
                                        };
                                        vt07 = vt07.max(v);
                                    }
                                }
                                for ai in &trn.muns {
                                    let mu = &eg.muni[*ai].key;
                                    if let Some((p, a)) = mu_dn.get(mu) {
                                        let a = a / 1_000f32;
                                        let pd = p / a * 2.5f32;
                                        let v = match pd {
                                            0f32..15f32 => 6f32,
                                            15f32..30f32 => 7f32,
                                            30f32..70f32 => 8f32,
                                            70f32..200f32 => 9f32,
                                            _ => 10f32,
                                        };
                                        vt07 = vt07.max(v);
                                    }
                                }

                                // transformer data finish
                                let mut tr_as = PeaAssVar::from(trn.n1d);
                                tr_as.arid = aid.to_string();
                                tr_as.pvid = pid.to_string();
                                tr_as.sbid = sid.to_string();
                                tr_as.fdid = fid.to_string();
                                tr_as.own = trn.own.to_string();
                                tr_as.peano =
                                    trn.tr_pea.clone().unwrap_or("".to_string()).to_string();
                                tr_as.aoj = aoj;
                                tr_as.aojcd = aojcd;
                                tr_as.aojs = aojs;
                                tr_as.v[VarType::None as usize].v = 0f32;
                                tr_as.v[VarType::NewCarReg as usize].v = vp01;
                                tr_as.v[VarType::Gpp as usize].v = vp02;
                                tr_as.v[VarType::MaxPosPowSub as usize].v = vs01;
                                tr_as.v[VarType::MaxNegPowSub as usize].v = vs02;
                                tr_as.v[VarType::VsppMv as usize].v = vs03;
                                tr_as.v[VarType::SppHv as usize].v = vs04;
                                tr_as.v[VarType::BigLotMv as usize].v = vs05;
                                tr_as.v[VarType::BigLotHv as usize].v = vs06;
                                tr_as.v[VarType::SubPowCap as usize].v = vs07;
                                tr_as.v[VarType::MaxPosPowFeeder as usize].v = vf01;
                                let pow_tr_sat = vs01 / trf_kva_2_kw(z2o(vs07));
                                tr_as.v[VarType::PowTrSat as usize].v = pow_tr_sat;

                                tr_as.v[VarType::MaxNegPowFeeder as usize].v = vf02;
                                tr_as.v[VarType::MaxPosDiffFeeder as usize].v = vf03;
                                tr_as.v[VarType::MaxNegDiffFeeder as usize].v = vf04;
                                tr_as.v[VarType::NoMeterTrans as usize].v = vt01;
                                tr_as.v[VarType::SmallSellTr as usize].v = vt02;
                                tr_as.v[VarType::ChgStnCapTr as usize].v = vt03;
                                tr_as.v[VarType::ChgStnSellTr as usize].v = vt04;
                                tr_as.v[VarType::PwCapTr as usize].v = vt05;
                                tr_as.v[VarType::ZoneTr as usize].v = vt06;
                                tr_as.v[VarType::PopTr as usize].v = vt07;
                                tr_as.v[VarType::UnbalPowTr as usize].v = vt08;
                                tr_as.v[VarType::PkPowTr as usize].v = vt09;
                                tr_as.v[VarType::LargeSellTr as usize].v = vt10;
                                tr_as.v[VarType::AllNoMeterTr as usize].v = vt11;
                                tr_as.v[VarType::NoMet1Ph as usize].v = nom1p;
                                tr_as.v[VarType::NoMet3Ph as usize].v = nom3p;
                                tr_as.v[VarType::NoTr as usize].v = vt12;
                                tr_as.v[VarType::EnGrowth as usize].v = grw;
                                if trn.own == "P" {
                                    tr_as.v[VarType::NoPeaTr as usize].v = vt12;
                                } else {
                                    tr_as.v[VarType::NoCusTr as usize].v = vt12;
                                }
                                tr_as.v[VarType::PkSelPowPhsAKw as usize].v = v_phs_a;
                                tr_as.v[VarType::PkSelPowPhsBKw as usize].v = v_phs_b;
                                tr_as.v[VarType::PkSelPowPhsCKw as usize].v = v_phs_c;
                                tr_as.v[VarType::PkSelPowPhsAvg as usize].v = v_ph_av;
                                tr_as.v[VarType::PkSelPowPhsMax as usize].v = v_ph_mx;
                                tr_as.v[VarType::UnbalPowRate as usize].v = v_ph_rt;
                                tr_as.v[VarType::TransLossKw as usize].v = v_loss;
                                tr_as.v[VarType::UnbalPowLossKw as usize].v = v_unba;
                                tr_as.v[VarType::CntTrUnbalLoss as usize].v = v_unb_cnt;
                                tr_as.v[VarType::CntTrSatLoss as usize].v = v_max_cnt;
                                tr_as.v[VarType::EvCarLikely as usize].v = evlk;
                                tr_as.v[VarType::SelectLikely as usize].v = selk;
                                tr_as.v[VarType::SolarEnergy as usize].v = solar;
                                tr_as.v[VarType::AllSellTr.tousz()].v = allsel;

                                if let Ok(mut tras_mx1) = c_tras_mx1.lock() {
                                    tras_mx1.max(&tr_as);
                                }
                                //tr_as.v[VarType::OfficeCovWg.tousz()].v = aojs.len();
                                //s_tr_sum.add(&tr_as);
                                s_tr_ass.push(tr_as);
                            } // end trans loop
                        } // end feeder loop
                        if let Ok(mut assv) = c_assv.lock() {
                            assv.append(&mut s_tr_ass);
                        }
                        /*
                        let bin: Vec<u8> =
                            bincode::encode_to_vec(&s_tr_ass, bincode::config::standard()).unwrap();
                        std::fs::write(format!("{dnm}/{sid}-raw.bin"), bin).unwrap();
                        */
                    } // end sub loop
                } // end provi loop

                //let mut aoj_m = HashMap::<usize, usize>::new();
                ////////////////////////////////
                ////////////////////////////////
            });
        } // end area
    });

    Ok(())
}

use std::sync::mpsc;
use std::thread;

pub fn stage_02_b(assrw1: Vec<PeaAssVar>, tras_mx1: &PeaAssVar) -> Result<(), Box<dyn Error>> {
    let cn = 10;
    //let sz = (assrw1.len() + cn - 1) / cn;
    let sz = assrw1.len().div_ceil(cn);

    //
    //////////////////////////////////////////////
    // EV Weight
    let mut we_ev = PeaAssVar::from(0u64);
    for (vt, vv) in WE_EV {
        we_ev.v[vt.tousz()].v = vv;
    }

    //////////////////////////////////////////////
    // Solar Weight
    let mut we_so = PeaAssVar::from(0u64);
    for (vt, vv) in WE_RE {
        we_so.v[vt.tousz()].v = vv;
    }

    //////////////////////////////////////////////
    // ETruck Weight
    let mut we_et = PeaAssVar::from(0u64);
    for (vt, vv) in WE_ET {
        we_et.v[vt.tousz()].v = vv;
    }

    //////////////////////////////////////////////
    // EV bike Weight
    let mut we_eb = PeaAssVar::from(0u64);
    for (vt, vv) in WE_EB {
        we_eb.v[vt.tousz()].v = vv;
    }

    let mut tras_raw = assrw1.clone();
    let mut tras_nor = tras_raw.clone();

    println!("======== STG2 ==== start nor");
    thread::scope(|s| {
        for tras_nor in tras_nor.chunks_mut(sz) {
            let tras_mx1 = tras_mx1.clone();
            s.spawn(move || {
                for tras in tras_nor {
                    tras.nor(&tras_mx1);
                }
            });
        }
    });
    {
        println!("======== STG2 ==== start ev");
        let mut tras_ev = tras_nor.clone();
        thread::scope(|s| {
            for (evs, rws) in tras_ev.chunks_mut(sz).zip(tras_raw.chunks_mut(sz)) {
                let we_ev = we_ev.clone();
                s.spawn(move || {
                    for (ev, rw) in evs.iter_mut().zip(rws.iter_mut()) {
                        ev.weigh(&we_ev);
                        ev.sum();
                        rw.v[VarType::HmChgEvTr as usize].v = ev.res;
                    }
                });
            }
        });
    }

    {
        println!("======== STG2 ==== start et");
        let mut tras_et = tras_nor.clone();
        thread::scope(|s| {
            for (ets, rws) in tras_et.chunks_mut(sz).zip(tras_raw.chunks_mut(sz)) {
                let we_et = we_et.clone();
                s.spawn(move || {
                    for (et, rw) in ets.iter_mut().zip(rws.iter_mut()) {
                        et.weigh(&we_et);
                        et.sum();
                        rw.v[VarType::ChgEtTr as usize].v = et.res;
                    }
                });
            }
        });
    }

    {
        println!("======== STG2 ==== start eb");
        let mut tras_eb = tras_nor.clone();
        thread::scope(|s| {
            for (ebs, rws) in tras_eb.chunks_mut(sz).zip(tras_raw.chunks_mut(sz)) {
                let we_eb = we_eb.clone();
                s.spawn(move || {
                    for (eb, rw) in ebs.iter_mut().zip(rws.iter_mut()) {
                        eb.weigh(&we_eb);
                        eb.sum();
                        rw.v[VarType::ChgEbTr as usize].v = eb.res;
                    }
                });
            }
        });
    }

    {
        println!("======== STG2 ==== start so");
        let mut tras_so = tras_nor.clone();
        thread::scope(|s| {
            for (sos, rws) in tras_so.chunks_mut(sz).zip(tras_raw.chunks_mut(sz)) {
                let we_so = we_so.clone();
                s.spawn(move || {
                    for (so, rw) in sos.iter_mut().zip(rws.iter_mut()) {
                        so.weigh(&we_so);
                        so.sum();
                        rw.v[VarType::SolarRoof as usize].v = so.res;
                    }
                });
            }
        });
    }

    println!("======== STG2 ==== SUM");
    let (tx0, rx) = mpsc::channel();
    let mut txv = vec![];
    for _ in 1..cn {
        txv.push(tx0.clone());
    }
    txv.push(tx0);
    thread::scope(|s| {
        for tras_raw in tras_raw.chunks_mut(sz) {
            let tx = txv.pop().unwrap();
            //let tx = tx0.clone();
            s.spawn(move || {
                let mut sum = PeaAssVar::from(0u64);
                for tras in tras_raw.iter() {
                    sum.add(tras);
                }
                tx.send(sum).unwrap();
            });
        }
    });

    let mut sum = PeaAssVar::from(0u64);
    for (i, r) in rx.iter().enumerate() {
        sum.add(&r);
    }

    println!("======== STG2 ==== MAX2");
    let (tx0, rx) = mpsc::channel();
    let mut txv = vec![];
    for _ in 1..cn {
        txv.push(tx0.clone());
    }
    txv.push(tx0);
    thread::scope(|s| {
        for tras_raw in tras_raw.chunks_mut(sz) {
            let tx = txv.pop().unwrap();
            s.spawn(move || {
                let mut max2 = PeaAssVar::from(0u64);
                for tr in tras_raw {
                    tr.v[VarType::LvPowSatTr as usize].v =
                        tr.v[VarType::PkPowTr as usize].v / z2o(tr.v[VarType::PwCapTr as usize].v);
                    tr.v[VarType::CntLvPowSatTr as usize].v =
                        if tr.v[VarType::LvPowSatTr as usize].v > 0.8f32 {
                            1f32
                        } else {
                            0f32
                        };
                    tr.v[VarType::ChgStnCap as usize].v = tr.v[VarType::ChgStnCapTr as usize].v;
                    tr.v[VarType::ChgStnSell as usize].v = tr.v[VarType::ChgStnSellTr as usize].v;
                    tr.v[VarType::MvPowSatTr as usize].v = tr.v[VarType::MaxPosPowSub as usize].v
                        / z2o(tr.v[VarType::SubPowCap as usize].v);
                    tr.v[VarType::MvVspp as usize].v = tr.v[VarType::VsppMv as usize].v;
                    tr.v[VarType::HvSpp as usize].v = tr.v[VarType::SppHv as usize].v;
                    tr.v[VarType::SmallSell as usize].v = tr.v[VarType::SmallSellTr as usize].v;
                    tr.v[VarType::LargeSell as usize].v = tr.v[VarType::LargeSellTr as usize].v;
                    tr.v[VarType::UnbalPow as usize].v = tr.v[VarType::UnbalPowTr as usize].v;
                    let v = tr.v[VarType::UnbalPowTr as usize].v
                        / z2o(tr.v[VarType::PwCapTr as usize].v);
                    tr.v[VarType::CntUnbalPow as usize].v = if v > 0.5f32 { 1f32 } else { 0f32 };

                    max2.max(tr);
                }
                let _ = tx.send(max2);
            });
        }
    });
    let mut max2 = PeaAssVar::from(0u64);
    //let mut sum2 = PeaAssVar::from(0u64);
    for (i, r) in rx.iter().enumerate() {
        max2.add(&r);
    }

    //////////////////////////////////////
    println!("======== STG2 ==== END");
    let mut we_uc1 = PeaAssVar::from(0u64);
    for (vt, vv) in WE_UC1 {
        we_uc1.v[vt.tousz()].v = vv;
    }
    let mut we_uc2 = PeaAssVar::from(0u64);
    for (vt, vv) in WE_UC2 {
        we_uc2.v[vt.tousz()].v = vv;
    }
    let mut we_uc3 = PeaAssVar::from(0u64);
    for (vt, vv) in WE_UC3 {
        we_uc3.v[vt.tousz()].v = vv;
    }
    let evsc = ev_scurv();
    let resc = re_scurv();
    let etsc = et_scurv();
    let ebsc = eb_scurv();
    println!("evsc: {} resc: {}", evsc.len(), resc.len());

    let mut tras_nor = tras_raw.clone();

    println!("======== STG3 ==== start nor");
    thread::scope(|s| {
        for tras_nor in tras_nor.chunks_mut(sz) {
            let max2 = max2.clone();
            s.spawn(move || {
                for tras in tras_nor {
                    tras.nor(&max2);
                }
            });
        }
    });

    println!("======== STG3 ==== CALC RATIO ");
    let mut tras_sum = tras_raw.clone();
    thread::scope(|s| {
        //for trraw in tras_raw.chunks_mut(sz) {
        for (trsum, trraw) in tras_sum.chunks_mut(sz).zip(tras_raw.chunks_mut(sz)) {
            let evsc = evsc.clone();
            let etsc = etsc.clone();
            let ebsc = ebsc.clone();
            let sum = sum.clone();
            s.spawn(move || {
                //for tras0 in trraw.iter_mut() {
                for (tras, tras0) in trsum.iter_mut().zip(trraw.iter_mut()) {
                    tras.nor(&sum);

                    //============================== EV consumption
                    tras0.v[VarType::NoHmChgEvTr as usize].v =
                        tras.v[VarType::HmChgEvTr as usize].v * 210_000f32;
                    tras0.v[VarType::PowHmChgEvTr as usize].v =
                        tras0.v[VarType::NoHmChgEvTr as usize].v * 0.007f32;
                    for (i, rt) in evsc.iter().enumerate() {
                        let evno = tras.v[VarType::HmChgEvTr.tousz()].v * EV_AT_2050 * rt;
                        tras0.vy[VarType::NoHmChgEvTr.tousz()].push(evno);
                        tras0.vy[VarType::PowHmChgEvTr.tousz()].push(evno * 0.007f32);
                        // ev car charger is 7kw
                        // everage charge 2 hour / day
                        // everage charge 1.5 hour / day
                        // everage charge 1.2 hour / day
                        // profit 0.42 baht per kwh
                        //
                        let evbt = if i < 0 {
                            0f32
                        } else {
                            evno * EV_CHG_POW_KW
                                * EV_DAY_CHG_HOUR
                                * EV_CHG_PROF_KW
                                * 365.0
                                * EV_CLAIM_RATE
                        };
                        tras0.vy[VarType::FirEvChgThb.tousz()].push(evbt);
                    }
                    tras0.v[VarType::FirEvChgThb.tousz()].v =
                        tras0.vy[VarType::FirEvChgThb.tousz()].iter().sum();

                    //============================== EV TRUCK consumption
                    // EV truck
                    for (i, rt) in etsc.iter().enumerate() {
                        let etno = tras.v[VarType::ChgEtTr.tousz()].v * ET_AT_2050 * rt;
                        tras0.vy[VarType::NoEtTr.tousz()].push(etno);
                        let etbt = if i < 0 {
                            0f32
                        } else {
                            etno * ET_CHG_POW_KW
                                * ET_DAY_CHG_HOUR
                                * EV_CHG_PROF_KW
                                * 365.0
                                * ET_CLAIM_RATE
                        };
                        tras0.vy[VarType::FirEtChgThb.tousz()].push(etbt);
                    }
                    tras0.v[VarType::FirEtChgThb.tousz()].v =
                        tras0.vy[VarType::FirEtChgThb.tousz()].iter().sum();

                    // EV bike
                    for (i, rt) in ebsc.iter().enumerate() {
                        let ebno = tras.v[VarType::ChgEbTr.tousz()].v * ET_AT_2050 * rt;
                        tras0.vy[VarType::NoEtTr.tousz()].push(ebno);
                        let ebbt = if i < 0 {
                            0f32
                        } else {
                            ebno * EB_CHG_POW_KW
                                * EB_DAY_CHG_HOUR
                                * EV_CHG_PROF_KW
                                * 365.0
                                * EB_CLAIM_RATE
                        };
                        tras0.vy[VarType::FirEbChgThb.tousz()].push(ebbt);
                    }
                    tras0.v[VarType::FirEbChgThb.tousz()].v =
                        tras0.vy[VarType::FirEbChgThb.tousz()].iter().sum();
                }
            });
            /*
                  //// save normal bin
                let bin: Vec<u8> =
                    bincode::encode_to_vec(&v_tras_nor, bincode::config::standard()).unwrap();
                std::fs::write(format!("{dnm}/{sid}-no2.bin"), bin).unwrap();
            */
        }
    });

    {
        println!("======== STG3 ==== USE CASE 1 ");
        let mut tras_uc1 = tras_nor.clone();
        thread::scope(|s| {
            for (truc1, trraw) in tras_uc1.chunks_mut(sz).zip(tras_raw.chunks_mut(sz)) {
                let we_uc1 = we_uc1.clone();
                s.spawn(move || {
                    for (tras, tras0) in truc1.iter_mut().zip(trraw.iter_mut()) {
                        tras.weigh(&we_uc1);
                        tras.sum();
                    }
                });
                /*
                //let bin: Vec<u8> =
                //    bincode::encode_to_vec(&v_uc1, bincode::config::standard()).unwrap();
                //std::fs::write(format!("{dnm}/{sid}-uc1.bin"), bin).unwrap();
                 */
            }
        });
    }
    {
        println!("======== STG3 ==== USE CASE 2 ");
        let mut tras_uc2 = tras_nor.clone();
        thread::scope(|s| {
            for (truc2, trraw) in tras_uc2.chunks_mut(sz).zip(tras_raw.chunks_mut(sz)) {
                let we_uc2 = we_uc2.clone();
                s.spawn(move || {
                    for (tras, tras0) in truc2.iter_mut().zip(trraw.iter_mut()) {
                        tras.weigh(&we_uc2);
                        tras.sum();
                        tras0.v[VarType::Uc2Val as usize].v = tras.res;
                    }
                });
            }
        });
    }
    {
        println!("======== STG3 ==== USE CASE 3 ");
        let mut tras_uc3 = tras_nor.clone();
        thread::scope(|s| {
            for (truc3, trraw) in tras_uc3.chunks_mut(sz).zip(tras_raw.chunks_mut(sz)) {
                let we_uc3 = we_uc3.clone();
                s.spawn(move || {
                    for (tras, tras0) in truc3.iter_mut().zip(trraw.iter_mut()) {
                        tras.weigh(&we_uc3);
                        tras.sum();
                        tras0.v[VarType::Uc3Val as usize].v = tras.res;
                    }
                });
            }
        });
    }

    {
        println!("======== STG4 ==== COST/BENEFIT ");
        thread::scope(|s| {
            for (i, trraw) in tras_raw.chunks_mut(sz).enumerate() {
                s.spawn(move || {
                    for tras in trraw.iter_mut() {
                        let ary = crate::ben2::ben_bill_accu(tras);
                        tras.vy[VarType::FirBilAccu.tousz()].append(&mut ary.clone());
                        tras.sum_yr(VarType::FirBilAccu);
                        //tras.v[VarType::FirBilAccu.tousz()].v = ary.iter().sum();

                        let csh = crate::ben2::ben_cash_flow(tras);
                        tras.vy[VarType::FirCashFlow.tousz()].append(&mut csh.clone());
                        tras.sum_yr(VarType::FirCashFlow);
                        //tras.v[VarType::FirCashFlow.tousz()].v = csh.iter().sum();

                        let drs = crate::ben2::ben_dr_save(tras);
                        tras.vy[VarType::FirDRSave.tousz()].append(&mut drs.clone());
                        tras.sum_yr(VarType::FirDRSave);
                        //tras.v[VarType::FirDRSave.tousz()].v = drs.iter().sum();

                        let bxc = crate::ben2::ben_boxline_save(tras);
                        tras.vy[VarType::FirMetBoxSave.tousz()].append(&mut bxc.clone());
                        tras.sum_yr(VarType::FirMetBoxSave);
                        //tras.v[VarType::FirMetBoxSave.tousz()].v = bxc.iter().sum();

                        let wks = crate::ben2::ben_work_save(tras);
                        tras.vy[VarType::FirLaborSave.tousz()].append(&mut wks.clone());
                        tras.sum_yr(VarType::FirLaborSave);
                        //tras.v[VarType::FirLaborSave.tousz()].v = wks.iter().sum();

                        let mts = crate::ben2::ben_sell_meter(tras);
                        tras.vy[VarType::FirMetSell.tousz()].append(&mut mts.clone());
                        tras.sum_yr(VarType::FirMetSell);
                        //tras.v[VarType::FirMetSell.tousz()].v = mts.iter().sum();

                        let ems = crate::ben2::ben_emeter(tras);
                        tras.vy[VarType::FirEMetSave.tousz()].append(&mut ems.clone());
                        tras.sum_yr(VarType::FirEMetSave);
                        //tras.v[VarType::FirEMetSave.tousz()].v = ems.iter().sum();

                        let mrs = crate::ben2::ben_mt_read(tras);
                        tras.vy[VarType::FirMetReadSave.tousz()].append(&mut mrs.clone());
                        tras.sum_yr(VarType::FirMetReadSave);
                        //tras.v[VarType::FirMetReadSave.tousz()].v = mrs.iter().sum();

                        let mds = crate::ben2::ben_mt_disconn(tras);
                        tras.vy[VarType::FirMetDisSave.tousz()].append(&mut mds.clone());
                        tras.sum_yr(VarType::FirMetDisSave);
                        //tras.v[VarType::FirMetDisSave.tousz()].v = mds.iter().sum();

                        let tos = crate::ben2::ben_tou_sell(tras);
                        tras.vy[VarType::FirTouSell.tousz()].append(&mut tos.clone());
                        tras.sum_yr(VarType::FirTouSell);
                        //tras.v[VarType::FirTouSell.tousz()].v = tos.iter().sum();

                        let trs = crate::ben2::ben_tou_read(tras);
                        tras.vy[VarType::FirTouReadSave.tousz()].append(&mut trs.clone());
                        tras.sum_yr(VarType::FirTouReadSave);
                        //tras.v[VarType::FirTouReadSave.tousz()].v = trs.iter().sum();

                        let tus = crate::ben2::ben_tou_update(tras);
                        tras.vy[VarType::FirTouUpdateSave.tousz()].append(&mut tus.clone());
                        tras.sum_yr(VarType::FirTouUpdateSave);
                        //tras.v[VarType::FirTouUpdateSave.tousz()].v = tus.iter().sum();

                        let ols = crate::ben2::ben_outage_labor(tras);
                        tras.vy[VarType::FirOutLabSave.tousz()].append(&mut ols.clone());
                        tras.sum_yr(VarType::FirOutLabSave);
                        //tras.v[VarType::FirOutLabSave.tousz()].v = ols.iter().sum();

                        let cps = crate::ben2::ben_reduce_complain(tras);
                        tras.vy[VarType::FirComplainSave.tousz()].append(&mut cps.clone());
                        tras.sum_yr(VarType::FirComplainSave);
                        //tras.v[VarType::FirComplainSave.tousz()].v = cps.iter().sum();

                        let asv = crate::ben2::ben_asset_value(tras);
                        tras.vy[VarType::FirAssetValue.tousz()].append(&mut asv.clone());
                        tras.sum_yr(VarType::FirAssetValue);
                        //tras.v[VarType::FirAssetValue.tousz()].v = asv.iter().sum();

                        let mes = crate::ben2::ben_model_entry(tras);
                        tras.vy[VarType::FirDataEntrySave.tousz()].append(&mut mes.clone());
                        tras.sum_yr(VarType::FirDataEntrySave);
                        //tras.v[VarType::FirDataEntrySave.tousz()].v = mes.iter().sum();

                        let dum = vec![0f32; 15];
                        tras.vy[VarType::FirBatSubSave.tousz()].append(&mut dum.clone());
                        tras.sum_yr(VarType::FirBatSubSave);
                        tras.vy[VarType::FirBatSvgSave.tousz()].append(&mut dum.clone());
                        tras.sum_yr(VarType::FirBatSvgSave);
                        tras.vy[VarType::FirBatEnerSave.tousz()].append(&mut dum.clone());
                        tras.sum_yr(VarType::FirBatEnerSave);
                        tras.vy[VarType::FirBatPriceDiff.tousz()].append(&mut dum.clone());
                        tras.sum_yr(VarType::FirBatPriceDiff);

                        let nome1 = tras.v[VarType::NoMet1Ph.tousz()].v;
                        let nome3 = tras.v[VarType::NoMet3Ph.tousz()].v;
                        let notr = tras.v[VarType::NoPeaTr.tousz()].v;
                        let nobess = 0.0;
                        //let bescap = 0.0;
                        let nodev = nome1 + nome3 + notr + nobess;

                        let bescap = tras.v[VarType::PowHmChgEvTr.tousz()].v * BESS_EVPOW_MWH_MULT;
                        tras.v[VarType::BessMWh.tousz()].v = bescap;

                        tras.v[VarType::NoDevice.tousz()].v = nodev;

                        tras.vy[VarType::CstMet1pIns.tousz()].append(&mut cst_m1p_ins(nome1));
                        tras.sum_yr(VarType::CstMet1pIns);
                        tras.vy[VarType::CstMet3pIns.tousz()].append(&mut cst_m3p_ins(nome3));
                        tras.sum_yr(VarType::CstMet3pIns);
                        tras.vy[VarType::CstTrIns.tousz()].append(&mut cst_tr_ins(notr));
                        tras.sum_yr(VarType::CstTrIns);
                        tras.vy[VarType::CstBessIns.tousz()].append(&mut cst_bes_ins(bescap));
                        tras.sum_yr(VarType::CstBessIns);
                        tras.vy[VarType::CstPlfmIns.tousz()].append(&mut cst_plfm_ins(nodev));
                        tras.sum_yr(VarType::CstPlfmIns);
                        tras.vy[VarType::CstCommIns.tousz()].append(&mut cst_comm_ins(nodev));
                        tras.sum_yr(VarType::CstCommIns);

                        tras.vy[VarType::CstMet1pImp.tousz()].append(&mut cst_m1p_imp(nome1));
                        tras.sum_yr(VarType::CstMet1pImp);
                        tras.vy[VarType::CstMet3pImp.tousz()].append(&mut cst_m3p_imp(nome3));
                        tras.sum_yr(VarType::CstMet3pImp);
                        tras.vy[VarType::CstTrImp.tousz()].append(&mut cst_tr_imp(notr));
                        tras.sum_yr(VarType::CstTrImp);
                        tras.vy[VarType::CstBessImp.tousz()].append(&mut cst_bes_imp(bescap));
                        tras.sum_yr(VarType::CstBessImp);
                        tras.vy[VarType::CstPlfmImp.tousz()].append(&mut cst_plfm_imp(nodev));
                        tras.sum_yr(VarType::CstPlfmImp);
                        tras.vy[VarType::CstCommImp.tousz()].append(&mut cst_comm_imp(nodev));
                        tras.sum_yr(VarType::CstCommImp);

                        tras.vy[VarType::CstMet1pOp.tousz()].append(&mut cst_m1p_op(nome1));
                        tras.sum_yr(VarType::CstMet1pOp);
                        tras.vy[VarType::CstMet3pOp.tousz()].append(&mut cst_m3p_op(nome3));
                        tras.sum_yr(VarType::CstMet3pOp);
                        tras.vy[VarType::CstTrOp.tousz()].append(&mut cst_tr_op(notr));
                        tras.sum_yr(VarType::CstTrOp);
                        tras.vy[VarType::CstBessOp.tousz()].append(&mut cst_bes_op(bescap));
                        tras.sum_yr(VarType::CstBessOp);
                        tras.vy[VarType::CstPlfmOp.tousz()].append(&mut cst_plfm_op(nodev));
                        tras.sum_yr(VarType::CstPlfmOp);
                        tras.vy[VarType::CstCommOp.tousz()].append(&mut cst_comm_op(nodev));
                        tras.sum_yr(VarType::CstCommOp);

                        let sel = tras.v[VarType::AllSellTr.tousz()].v;

                        tras.vy[VarType::EirCustLossSave.tousz()]
                            .append(&mut eir_cust_loss_save(sel));
                        tras.sum_yr(VarType::EirCustLossSave);
                        tras.vy[VarType::EirConsumSave.tousz()].append(&mut eir_cust_save(sel));
                        tras.sum_yr(VarType::EirConsumSave);
                        tras.vy[VarType::EirGrnHsEmsSave.tousz()].append(&mut eir_ghg_save(sel));
                        tras.sum_yr(VarType::EirGrnHsEmsSave);
                        tras.vy[VarType::EirCustMvRev.tousz()].append(&mut eir_cust_mv_rev(sel));
                        tras.sum_yr(VarType::EirCustMvRev);
                        tras.vy[VarType::EirCustEvSave.tousz()].append(&mut eir_cust_ev_save(sel));
                        tras.sum_yr(VarType::EirCustEvSave);
                        tras.vy[VarType::EirCustEtrkSave.tousz()]
                            .append(&mut eir_cust_etruck_save(sel));
                        tras.sum_yr(VarType::EirCustEtrkSave);
                        tras.vy[VarType::EirSolaRfTopSave.tousz()]
                            .append(&mut eir_cust_solar_roof(sel));
                        tras.sum_yr(VarType::EirSolaRfTopSave);
                        tras.vy[VarType::EirEnerResvSave.tousz()].append(&mut eir_en_rev_save(sel));
                        tras.sum_yr(VarType::EirEnerResvSave);

                        /*
                        tras.v[VarType::CstMet1pIns.tousz()].v =
                            tras.vy[VarType::CstMet1pIns.tousz()].iter().sum();

                        tras.v[VarType::CstMet1pIns.tousz()].v =
                            tras.vy[VarType::CstMet1pIns.tousz()].iter().sum();

                        tras.v[VarType::CstMet1pIns.tousz()].v =
                            tras.vy[VarType::CstMet1pIns.tousz()].iter().sum();

                        tras.v[VarType::CstMet1pIns.tousz()].v =
                            tras.vy[VarType::CstMet1pIns.tousz()].iter().sum();

                        tras.v[VarType::CstMet1pIns.tousz()].v =
                            tras.vy[VarType::CstMet1pIns.tousz()].iter().sum();

                        tras.v[VarType::CstMet1pIns.tousz()].v =
                            tras.vy[VarType::CstMet1pIns.tousz()].iter().sum();
                        tras.v[VarType::CstMet3pIns.tousz()].v =
                            tras.vy[VarType::CstMet3pIns.tousz()].iter().sum();
                        tras.v[VarType::CstTrIns.tousz()].v =
                            tras.vy[VarType::CstTrIns.tousz()].iter().sum();
                        tras.v[VarType::CstBessIns.tousz()].v =
                            tras.vy[VarType::CstBessIns.tousz()].iter().sum();
                        tras.v[VarType::CstPlfmIns.tousz()].v =
                            tras.vy[VarType::CstPlfmIns.tousz()].iter().sum();
                        tras.v[VarType::CstCommIns.tousz()].v =
                            tras.vy[VarType::CstCommIns.tousz()].iter().sum();

                        tras.v[VarType::CstMet1pImp.tousz()].v =
                            tras.vy[VarType::CstMet1pImp.tousz()].iter().sum();
                        tras.v[VarType::CstMet3pImp.tousz()].v =
                            tras.vy[VarType::CstMet3pImp.tousz()].iter().sum();
                        tras.v[VarType::CstTrImp.tousz()].v =
                            tras.vy[VarType::CstTrImp.tousz()].iter().sum();
                        tras.v[VarType::CstBessImp.tousz()].v =
                            tras.vy[VarType::CstBessImp.tousz()].iter().sum();
                        tras.v[VarType::CstPlfmImp.tousz()].v =
                            tras.vy[VarType::CstPlfmImp.tousz()].iter().sum();
                        tras.v[VarType::CstCommImp.tousz()].v =
                            tras.vy[VarType::CstCommImp.tousz()].iter().sum();

                        tras.v[VarType::CstMet1pOp.tousz()].v =
                            tras.vy[VarType::CstMet1pOp.tousz()].iter().sum();
                        tras.v[VarType::CstMet3pOp.tousz()].v =
                            tras.vy[VarType::CstMet3pOp.tousz()].iter().sum();
                        tras.v[VarType::CstTrOp.tousz()].v =
                            tras.vy[VarType::CstTrOp.tousz()].iter().sum();
                        tras.v[VarType::CstBessOp.tousz()].v =
                            tras.vy[VarType::CstBessOp.tousz()].iter().sum();
                        tras.v[VarType::CstPlfmOp.tousz()].v =
                            tras.vy[VarType::CstPlfmOp.tousz()].iter().sum();
                        tras.v[VarType::CstCommOp.tousz()].v =
                            tras.vy[VarType::CstCommOp.tousz()].iter().sum();

                        tras.v[VarType::EirCustLossSave.tousz()].v =
                            tras.vy[VarType::EirCustLossSave.tousz()].iter().sum();
                        tras.v[VarType::EirConsumSave.tousz()].v =
                            tras.vy[VarType::EirConsumSave.tousz()].iter().sum();
                        tras.v[VarType::EirGrnHsEmsSave.tousz()].v =
                            tras.vy[VarType::EirGrnHsEmsSave.tousz()].iter().sum();
                        tras.v[VarType::EirCustMvRev.tousz()].v =
                            tras.vy[VarType::EirCustMvRev.tousz()].iter().sum();
                        tras.v[VarType::EirCustEvSave.tousz()].v =
                            tras.vy[VarType::EirCustEvSave.tousz()].iter().sum();
                        tras.v[VarType::EirCustEtrkSave.tousz()].v =
                            tras.vy[VarType::EirCustEtrkSave.tousz()].iter().sum();
                        tras.v[VarType::EirSolaRfTopSave.tousz()].v =
                            tras.vy[VarType::EirSolaRfTopSave.tousz()].iter().sum();
                        tras.v[VarType::EirEnerResvSave.tousz()].v =
                            tras.vy[VarType::EirEnerResvSave.tousz()].iter().sum();
                        */

                        ass_calc(tras).expect("?");
                    }
                });
            }
        });
    }

    println!("======== STG4 ==== SAVE SUB FILES ");
    let mut tr_sb_m = HashMap::<String, Vec<PeaAssVar>>::new();
    for trraw in tras_raw {
        let sbid = trraw.sbid.to_string();
        //let assv = tr_sb_m.entry(sbid).or_insert_with(Vec::<PeaAssVar>::new);
        let assv = tr_sb_m.entry(sbid).or_default();
        assv.push(trraw);
    }
    for (k, v) in tr_sb_m {
        let bin: Vec<u8> = bincode::encode_to_vec(&v, bincode::config::standard()).unwrap();
        let fnm = format!("{DNM}/{k}-rw4.bin");
        std::fs::write(fnm, bin).unwrap();
    }

    Ok(())
}

pub fn ass_calc(sbas: &mut PeaAssVar) -> Result<(), Box<dyn Error>> {
    // ==========  LOSS CALCULATION
    let unb_los = sbas.v[VarType::UnbalPowLossKw.tousz()].v
        * UNBAL_HOUR_PER_DAY
        * SAVE_LOSS_UNIT_PRICE
        * UNBAL_CALC_FACTOR
        * 365.0;
    //let unb_los = sbas.v[VarType::UnbalPowLossKw.tousz()].v * 4.0 * 4.0;
    //
    // claim save ratio 0.5
    let mut los_sav = unb_los * UNBAL_LOSS_CLAIM_RATE;
    //
    // transformer may die within 5 years
    // unit price for replace transformers
    // claim save ratio 0.5
    let mut tr_sav = sbas.v[VarType::CntTrSatLoss.tousz()].v / TRANS_REPL_WITHIN_YEAR
        * TRANS_REPL_UNIT_PRICE
        * TRANS_REPL_CLAIM_RATE;
    let mut ubt_sav = sbas.v[VarType::CntTrUnbalLoss.tousz()].v / TRANS_REPL_WITHIN_YEAR
        * TRANS_REPL_UNIT_PRICE
        * UNBAL_REPL_CLAIM_RATE;
    let mut all_sel = sbas.v[VarType::AllSellTr.tousz()].v
                    * NON_TECH_LOSS_RATIO
                    * 12.0 // in one year
                    * SAVE_LOSS_UNIT_PRICE
                    * NOTEC_LOSS_CLAIM_RATE;
    //sbas.v[VarType::AllSellTr.tousz()].v * 12.0 * 0.9 * 4_000f32 * 0.01;

    sbas.vy[VarType::FirUnbSave.tousz()].retain(|&_| false);
    sbas.vy[VarType::FirTrSatSave.tousz()].retain(|&_| false);
    sbas.vy[VarType::FirTrPhsSatSave.tousz()].retain(|&_| false);
    sbas.vy[VarType::FirNonTechLoss.tousz()].retain(|&_| false);
    for i in 0..15 {
        los_sav *= 1.0 + ENERGY_GRW_RATE;
        tr_sav *= 1.0 + ENERGY_GRW_RATE;
        ubt_sav *= 1.0 + ENERGY_GRW_RATE;
        all_sel *= 1.0 + ENERGY_GRW_RATE;
        //all_sel = 0.0;
        let (los, tr, ubt, all) = if i < 3 {
            (0.0, 0.0, 0.0, 0.0)
        } else {
            (los_sav, tr_sav, ubt_sav, all_sel)
        };
        sbas.vy[VarType::FirUnbSave.tousz()].push(los);
        sbas.vy[VarType::FirTrSatSave.tousz()].push(tr);
        sbas.vy[VarType::FirTrPhsSatSave.tousz()].push(ubt);
        sbas.vy[VarType::FirNonTechLoss.tousz()].push(all);
    }
    //sbas.v[VarType::FirUnbSave.tousz()].v = sbas.vy[VarType::FirUnbSave.tousz()].iter().sum();
    sbas.sum_yr(VarType::FirUnbSave);
    //sbas.v[VarType::FirTrSatSave.tousz()].v = sbas.vy[VarType::FirTrSatSave.tousz()].iter().sum();
    sbas.sum_yr(VarType::FirTrSatSave);
    //sbas.v[VarType::FirTrPhsSatSave.tousz()].v =
    //    sbas.vy[VarType::FirTrPhsSatSave.tousz()].iter().sum();
    sbas.sum_yr(VarType::FirTrPhsSatSave);
    //sbas.v[VarType::FirNonTechLoss.tousz()].v =
    //    sbas.vy[VarType::FirNonTechLoss.tousz()].iter().sum();
    sbas.sum_yr(VarType::FirNonTechLoss);

    let mut fir_cpx_opx: Vec<f32> = vec![0f32; 15];
    let mut eir_cpx_opx: Vec<f32> = vec![0f32; 15];

    // CAPOPEX
    let mut capop: Vec<f32> = vec![0f32; 15];

    // CAPEX
    sbas.v[VarType::CstCapEx.tousz()].v = CAPEX_FLDS.iter().map(|vt| sbas.v[vt.tousz()].v).sum();
    let mut vy0: Vec<f32> = vec![0f32; 15];
    for vt in &CAPEX_FLDS {
        for (i, vy) in sbas.vy[vt.tousz()].iter().enumerate() {
            vy0[i] += vy;
            capop[i] += vy;
            fir_cpx_opx[i] -= vy;
            eir_cpx_opx[i] -= vy;
        }
    }
    sbas.vy[VarType::CstCapEx.tousz()] = vy0;

    let reinv = sbas.v[VarType::CstCapEx.tousz()].v * REINVEST_RATE;

    sbas.vy[VarType::CstReinvest.tousz()].retain(|&_| false);
    sbas.vy[VarType::CstReinvest.tousz()].append(&mut cst_reinvest(reinv));
    //sbas.v[VarType::CstReinvest.tousz()].v = sbas.vy[VarType::CstReinvest.tousz()].iter().sum();
    sbas.sum_yr(VarType::CstReinvest);

    // OPEX
    sbas.v[VarType::CstOpEx.tousz()].v = OPEX_FLDS.iter().map(|vt| sbas.v[vt.tousz()].v).sum();
    let mut vy0: Vec<f32> = vec![0f32; 15];
    for vt in &OPEX_FLDS {
        for (i, vy) in sbas.vy[vt.tousz()].iter().enumerate() {
            vy0[i] += vy;
            capop[i] += vy;
            fir_cpx_opx[i] -= vy;
            eir_cpx_opx[i] -= vy;
        }
    }
    sbas.vy[VarType::CstOpEx.tousz()] = vy0;

    sbas.v[VarType::CstCapOpEx.tousz()].v = sbas.v[VarType::CstOpEx.tousz()].v
        + sbas.v[VarType::CstCapEx.tousz()].v
        + sbas.v[VarType::CstReinvest.tousz()].v;
    sbas.vy[VarType::CstCapOpEx.tousz()] = capop;

    // FIR
    sbas.v[VarType::FirSum.tousz()].v = FIR_FLDS.iter().map(|vt| sbas.v[vt.tousz()].v).sum();
    let mut vy0: Vec<f32> = vec![0f32; 15];
    for vt in &FIR_FLDS {
        if sbas.vy[vt.tousz()].len() > 15 {
            //println!("exceed {vt:?} = {}", sbas.vy[vt.tousz()].len());
        }
        for (i, vy) in sbas.vy[vt.tousz()].iter().take(15).enumerate() {
            vy0[i] += vy;
            fir_cpx_opx[i] += vy;
        }
    }
    sbas.vy[VarType::FirSum.tousz()] = vy0;

    // EIR
    sbas.v[VarType::EirSum.tousz()].v = EIR_FLDS.iter().map(|vt| sbas.v[vt.tousz()].v).sum();
    let mut vy0: Vec<f32> = vec![0f32; 15];
    for vt in &EIR_FLDS {
        for (i, vy) in sbas.vy[vt.tousz()].iter().enumerate() {
            vy0[i] += vy;
            eir_cpx_opx[i] -= vy;
        }
    }
    sbas.vy[VarType::EirSum.tousz()] = vy0;
    sbas.vy[VarType::FirCstRate.tousz()] = fir_cpx_opx.clone();
    sbas.vy[VarType::EirCstRate.tousz()] = eir_cpx_opx.clone();

    let fir_cpx_opx = fir_cpx_opx
        .iter()
        .filter(|n| !n.is_nan())
        .cloned()
        .collect::<Vec<_>>();
    let s0 = fir_cpx_opx.iter().sum::<f32>();

    let firr = if fir_cpx_opx.len() == 15 && s0 > 0f32 {
        let guess = Some(0.);
        let fir: Vec<f64> = fir_cpx_opx.iter().map(|n| *n as f64).collect();
        if let Ok(firr) = financial::irr(&fir, guess) {
            firr * 100.0f64
        } else {
            0f64
        }
    } else {
        0f64
    };
    sbas.v[VarType::FirCstRate.tousz()].v = firr as f32;

    let eir_cpx_opx = eir_cpx_opx
        .iter()
        .filter(|n| !n.is_nan())
        .cloned()
        .collect::<Vec<_>>();
    let s0 = eir_cpx_opx.iter().sum::<f32>();
    let eirr = if eir_cpx_opx.len() == 15 && s0 > 0f32 {
        let guess = Some(0.);
        let eir: Vec<f64> = eir_cpx_opx.iter().map(|n| *n as f64).collect();
        if let Ok(eirr) = financial::irr(&eir, guess) {
            eirr * 100.0f64
        } else {
            0f64
        }
    } else {
        0f64
    };
    //println!("FIRR: {}", firr);
    sbas.v[VarType::EirCstRate.tousz()].v = eirr as f32;

    Ok(())
}

//================================================
//================================================
//
