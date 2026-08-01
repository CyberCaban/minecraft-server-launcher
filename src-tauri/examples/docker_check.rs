use bollard::container::LogOutput;
use bollard::query_parameters::{ListContainersOptionsBuilder, LogsOptionsBuilder};
use bollard::Docker;
use futures_util::StreamExt;
use std::collections::HashMap;

const PROJECT: &str = "name";

fn main() {
    let rt = tokio::runtime::Runtime::new().expect("runtime");
    rt.block_on(async {
        let docker = match Docker::connect_with_local_defaults() {
            Ok(d) => {
                println!("[ok] connect_with_local_defaults");
                d
            }
            Err(e) => {
                println!("[FAIL] connect_with_local_defaults: {e}");
                return;
            }
        };

        match docker.ping().await {
            Ok(_) => println!("[ok] ping"),
            Err(e) => println!("[FAIL] ping: {e}"),
        }

        let mut filters = HashMap::new();
        filters.insert(
            "label".to_string(),
            vec![format!("com.docker.compose.project={PROJECT}")],
        );
        let opts = ListContainersOptionsBuilder::default()
            .all(true)
            .filters(&filters)
            .build();
        match docker.list_containers(Some(opts)).await {
            Ok(containers) => {
                if containers.is_empty() {
                    println!("[WARN] no containers for project={PROJECT}");
                }
                for c in containers {
                    println!(
                        "[ok] found container id={:?} names={:?} state={:?} status={:?}",
                        c.id.as_deref().map(|s| &s[..s.len().min(12)]),
                        c.names,
                        c.state,
                        c.status
                    );
                }
            }
            Err(e) => println!("[FAIL] list_containers: {e}"),
        }

        let running_opts = ListContainersOptionsBuilder::default()
            .filters(&filters)
            .build();
        let id = match docker.list_containers(Some(running_opts)).await {
            Ok(mut cs) => match cs.pop() {
                Some(c) => c.id.unwrap(),
                None => {
                    println!("[WARN] no running container, skipping logs/exec");
                    return;
                }
            },
            Err(e) => {
                println!("[FAIL] list_containers(running): {e}");
                return;
            }
        };
        println!("[ok] logs/exec target id={}", &id[..id.len().min(12)]);

        let log_opts = LogsOptionsBuilder::default()
            .stdout(true)
            .stderr(true)
            .tail("5")
            .build();
        let stream = docker.logs(&id, Some(log_opts));
        futures_util::pin_mut!(stream);
        let mut count = 0usize;
        while let Some(item) = stream.next().await {
            match item {
                Ok(LogOutput::StdOut { message }) | Ok(LogOutput::StdErr { message }) => {
                    let s = String::from_utf8_lossy(&message);
                    println!("[log] {}", s.trim());
                    count += 1;
                }
                Ok(_) => {}
                Err(e) => {
                    println!("[FAIL] logs stream error: {e}");
                    break;
                }
            }
        }
        println!("[ok] logs stream ended, {count} chunks");

        let created = match docker
            .create_exec(
                &id,
                bollard::exec::CreateExecOptions {
                    attach_stdout: Some(true),
                    attach_stderr: Some(true),
                    cmd: Some(vec!["echo".to_string(), "exec-ok".to_string()]),
                    ..Default::default()
                },
            )
            .await
        {
            Ok(info) => {
                println!("[ok] create_exec: {:?}", info.id);
                info
            }
            Err(e) => {
                println!("[FAIL] create_exec: {e}");
                return;
            }
        };

        match docker.start_exec(&created.id, None).await {
            Ok(result) => match result {
                bollard::exec::StartExecResults::Attached { output, .. } => {
                    let mut buf = Vec::new();
                    futures_util::pin_mut!(output);
                    while let Some(item) = output.next().await {
                        if let Ok(LogOutput::StdOut { message }) = item {
                            buf.extend_from_slice(&message);
                        }
                    }
                    println!(
                        "[ok] start_exec output: {}",
                        String::from_utf8_lossy(&buf)
                    );
                }
                bollard::exec::StartExecResults::Detached => {
                    println!("[ok] start_exec detached")
                }
            },
            Err(e) => println!("[FAIL] start_exec: {e}"),
        }
    });
}
