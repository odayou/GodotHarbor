fn main() {
    // A1-1: 支持 stdio（默认，给 CLI 用）和 tcp（给 harbor-bridge addon 用）两种 transport
    let mut transport = "stdio".to_string();
    let mut port: u16 = 0;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--transport" => {
                if let Some(t) = args.next() {
                    transport = t;
                }
            }
            "--port" => {
                if let Some(p) = args.next() {
                    port = p.parse().unwrap_or(0);
                }
            }
            "--help" | "-h" => {
                println!("harbor-mcp-server [options]");
                println!("  --transport stdio|tcp   传输方式（默认 stdio）");
                println!("  --port <N>              TCP 端口（transport=tcp 时必填）");
                return;
            }
            _ => {}
        }
    }

    match transport.as_str() {
        "tcp" => {
            if port == 0 {
                eprintln!("TCP transport 需指定 --port <N>");
                std::process::exit(1);
            }
            godot_harbor_lib::mcp::server::run_mcp_server_tcp(port);
        }
        "stdio" | _ => {
            godot_harbor_lib::mcp::server::run_mcp_server();
        }
    }
}
