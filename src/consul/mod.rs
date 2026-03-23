use std::time::Duration;
use anyhow::{anyhow};
use reqwest::Client;
use crate::Service;
use tokio::task::JoinSet;
use crate::structs::{ConsulEntry, ServiceIP};

pub async fn fetch_nodes(env: &str, services: Vec<&str>) -> anyhow::Result<Vec<Service>> {
    
    let consul_url = format!("http://nest-consul-{}.nest.r53.xcal.tv:8500/", env);

    let mut join_set = JoinSet::new();
    for service in services {
        join_set.spawn({
            let service_name = service.to_string();
            let cloned_consul_url = consul_url.clone();
            async move {
                get_consul_nodes(&cloned_consul_url, &service_name).await
            }
        });
    }

    let mut hosts = Vec::new();
    while let Some(joined) = join_set.join_next().await {
        match joined {
            Ok(Ok(nodes)) => hosts.push(nodes),
            Err(e) => return Err(anyhow!(" JointError : {} ",e)),
            Ok(Err(e)) => return Err(anyhow!(" Error : {}", e)),
        }
    }
    hosts.sort();
    Ok(hosts)
}

async fn get_consul_nodes(
    consul_url: &str,
    service_name: &str,
) -> anyhow::Result<Service> {
    let client = Client::builder()
        .connect_timeout(Duration::from_secs(1))
        //.timeout(Duration::from_secs(5))
        .build()
        .unwrap();

    let body: String = client.get(format!(
        "{}v1/health/service/{}",
        consul_url, service_name
    )).send().await?
    .text()
    .await?;

    let raws: Vec<ConsulEntry> = serde_json::from_str(&body)?;

    let ips: Vec<ServiceIP> = raws
        .into_iter()
        .map(|entry| ServiceIP {
            checked: false,
            ip: entry.node.address,
            checks: entry.checks.into_iter().map(|check| check.status).collect(),
        })
        .collect();

    Ok(Service {
        checked: false,
        service_name: service_name.to_string(),
        ips,
    })
}
