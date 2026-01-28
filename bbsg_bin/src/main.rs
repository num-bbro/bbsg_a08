use std::env;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let now = std::time::SystemTime::now();
    let a1 = env::args().nth(1).unwrap_or("?".to_string());
    let a2 = env::args().nth(2).unwrap_or("?".to_string());
    let a3 = env::args().nth(3).unwrap_or("docgen".to_string());
    match a1.as_str() {

        "X1" => bbsg_a08::stx1::stage_x1()?,
        "X2" => bbsg_a08::stx2::stage_02()?,
        "CM2" => bbsg_a08::utl3::excel_cmd2()?,
        "CM1" => bbsg_a08::utl3::excel_cmd1()?,
        "AR5" => {
            bbsg_a08::utl2::archi5(&a2, &a3)?;
        }
        "AR4" => {
            bbsg_a08::utl2::archi4(&a2, &a3)?;
        }
        "AR3" => {
            bbsg_a08::utl2::archi_vids(&a2)?;
        }
        "AR2" => {
            bbsg_a08::utl2::archi2(&a2)?;
        }
        "AR1" => {
            //bbsg_a08::utl2::archi1()?;
        }
        "14" => {
            bbsg_a08::stg1::stage_01()?;
            bbsg_a08::stg2::stage_02()?;
            bbsg_a08::stg3::stage_03()?;
            bbsg_a08::stg4::stage_04()?;
        }
        "24" => {
            bbsg_a08::stg2::stage_02()?;
            bbsg_a08::stg3::stage_03()?;
            bbsg_a08::stg4::stage_04()?;
        }
        "23" => {
            bbsg_a08::stg2::stage_02()?;
            bbsg_a08::stg3::stage_03()?;
        }
        "01" => bbsg_a08::stg1::stage_01()?,
        "02" => bbsg_a08::stg2::stage_02()?,
        "03" => bbsg_a08::stg3::stage_03()?,
        "04" => bbsg_a08::stg4::stage_04()?,
        "T04" => bbsg_a08::p09::check_sorf(),
        "T03" => bbsg_a08::p09::test_3(), // ECU2
        "T02" => bbsg_a08::p09::test_2(), // ECU1
        "T01" => bbsg_a08::p09::test_1(),
        "LPT" => bbsg_a08::utl::test_lp24()?,
        "SBL" => bbsg_a08::p09::sub_load()?,
        "web1" => bbsg_a08::p09::web1().await?,
        n => {
            println!("'{}' NG command", n);
        }
    }
    let se = now.elapsed().unwrap().as_secs();
    let mi = se / 60;
    println!("time {se} sec = {mi} min");
    Ok(())
}
