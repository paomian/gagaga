use core::cmp::Ordering;
use std::env;
use std::error::Error;
use std::fs;
use trust_dns_resolver::config::*;
use trust_dns_resolver::Resolver;
const CN_EMAIL_SUFFIX: [&str; 17] = [
    "qq.com.",
    "netease.com.",
    "sina.com.cn.",
    "163.com.",
    "126.com.",
    "dangdang.com.",
    "sohu.com.cn.",
    "aliyun.com.",
    "2980.com.",
    "baidu.com.",
    "alibaba-inc.com.",
    "jd.com.",
    "huawei.com.",
    "mxhichina.com.",
    "263xmail.com.",
    "139.com.",
    "outlook.cn.",
];
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let file = fs::File::open(&args[1])?;
    let email_column = args[2].parse::<usize>()?;
    println!("parse file:{},and the {}ed column is email.",&args[1],&args[2]);
    let file_name = args[1]
        .as_str()
        .splitn(2, '.')
        .collect::<Vec<&str>>()
        .first()
        .unwrap()
        .to_owned();
    let i18n_file_name = format!("{}_i18n.csv", file_name);
    let cn_file_name = format!("{}_cn.csv", file_name);
    let i18n_file = fs::File::create(&i18n_file_name)?;
    let cn_file = fs::File::create(&cn_file_name)?;
    let mut cn_wtr = csv::Writer::from_writer(cn_file);
    let mut i18n_wtr = csv::Writer::from_writer(i18n_file);
    let mut rdr = csv::Reader::from_reader(file);
    let headers = rdr.headers()?;
    cn_wtr.write_record(headers)?;
    i18n_wtr.write_record(headers)?;
    let resolver = Resolver::new(ResolverConfig::default(), ResolverOpts::default()).unwrap();
    for result in rdr.records() {
        let record = result?;
        let email = record
            .get(email_column - 1)
            .expect(&format!("email not found {:?}", record));
        let domain = email
            .split('@')
            .last()
            .expect(&format!("domain not found in {:?}", email));
        let response = resolver
            .mx_lookup(domain)
            .expect(&format!("{}:not resolve", domain));
        let min_mx_record = response
            .into_iter()
            .min_by(|x, y| {
                if x.preference() == y.preference() {
                    Ordering::Equal
                } else if x.preference() < y.preference() {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            })
            .unwrap();
        if CN_EMAIL_SUFFIX
            .into_iter()
            .any(|x| min_mx_record.exchange().to_utf8().ends_with(x))
        {
            cn_wtr.write_record(&record)?;
        } else {
            i18n_wtr.write_record(&record)?;
        }
    }
    Ok(())
}
