//! Tauri集成示例
//! 
//! 演示如何在Tauri应用中集成rat_quickdns和rat_quickmem
//! 注意：此示例仅展示代码结构，需要在实际Tauri项目中使用

// 以下代码应放在Tauri项目的src-tauri/src/dns.rs中

use rat_quickdns::{EasyDnsResolver, DnsQueryRequest, DnsQueryResponse, encode, decode};
use std::sync::Arc;
use tokio::sync::Mutex;

/// DNS解析器状态（全局单例）
pub struct DnsState {
    resolver: Arc<EasyDnsResolver>,
}

impl DnsState {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let resolver = EasyDnsResolver::quick_setup()?;
        Ok(Self {
            resolver: Arc::new(resolver),
        })
    }
}

/// Tauri命令：解析域名
#[tauri::command]
pub async fn resolve_domain(
    domain: String,
    record_type: String,
    state: tauri::State<'_, Arc<Mutex<DnsState>>>,
) -> Result<Vec<String>, String> {
    let state = state.lock().await;
    state.resolver
        .resolve_type(&domain, &record_type)
        .await
        .map_err(|e| e.to_string())
}

/// Tauri命令：处理二进制DNS查询（使用bincode2和rat_quickmem）
#[tauri::command]
pub async fn process_dns_query(
    query_data: Vec<u8>,
    state: tauri::State<'_, Arc<Mutex<DnsState>>>,
) -> Result<Vec<u8>, String> {
    let state = state.lock().await;
    state.resolver
        .process_encoded_query(&query_data)
        .await
        .map_err(|e| e.to_string())
}

/// Tauri命令：批量处理DNS查询
#[tauri::command]
pub async fn batch_resolve_domains(
    domains: Vec<String>,
    record_type: String,
    state: tauri::State<'_, Arc<Mutex<DnsState>>>,
) -> Result<Vec<Vec<String>>, String> {
    let state = state.lock().await;
    
    let mut results = Vec::with_capacity(domains.len());
    
    for domain in domains {
        match state.resolver.resolve_type(&domain, &record_type).await {
            Ok(ips) => results.push(ips),
            Err(e) => return Err(format!("解析 {} 失败: {}", domain, e)),
        }
    }
    
    Ok(results)
}

// 以下代码应放在Tauri项目的src-tauri/src/main.rs中

/*
use tauri::Manager;
use std::sync::Arc;
use tokio::sync::Mutex;

mod dns;
use dns::{DnsState, resolve_domain, process_dns_query, batch_resolve_domains};

#[tokio::main]
async fn main() {
    // 初始化DNS解析器
    let dns_state = Arc::new(Mutex::new(
        DnsState::new().expect("Failed to initialize DNS resolver")
    ));
    
    tauri::Builder::default()
        .manage(dns_state)
        .invoke_handler(tauri::generate_handler![
            resolve_domain,
            process_dns_query,
            batch_resolve_domains
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
*/

// 以下代码应放在Tauri项目的src/App.tsx或其他前端文件中

/*
import { invoke } from "@tauri-apps/api/tauri";
import { useState } from "react";

function App() {
  const [domain, setDomain] = useState("");
  const [recordType, setRecordType] = useState("A");
  const [results, setResults] = useState<string[]>([]);
  const [loading, setLoading] = useState(false);

  async function resolveDomain() {
    try {
      setLoading(true);
      const response = await invoke<string[]>("resolve_domain", {
        domain,
        recordType,
      });
      setResults(response);
    } catch (error) {
      console.error(error);
      setResults([`Error: ${error}`]);
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="container">
      <h1>DNS Resolver</h1>
      
      <div className="row">
        <input
          type="text"
          placeholder="Enter domain (e.g. example.com)"
          value={domain}
          onChange={(e) => setDomain(e.target.value)}
        />
        
        <select 
          value={recordType} 
          onChange={(e) => setRecordType(e.target.value)}
        >
          <option value="A">A</option>
          <option value="AAAA">AAAA</option>
          <option value="CNAME">CNAME</option>
          <option value="MX">MX</option>
          <option value="TXT">TXT</option>
        </select>
        
        <button onClick={resolveDomain} disabled={loading}>
          {loading ? "Resolving..." : "Resolve"}
        </button>
      </div>
      
      <div className="results">
        <h2>Results:</h2>
        {results.length > 0 ? (
          <ul>
            {results.map((result, index) => (
              <li key={index}>{result}</li>
            ))}
          </ul>
        ) : (
          <p>No results yet</p>
        )}
      </div>
    </div>
  );
}

export default App;
*/

// 实际运行示例（仅用于演示）
fn main() {
    println!("这是Tauri集成示例代码，请在实际Tauri项目中使用");
    println!("请参考代码注释了解如何集成到Tauri项目中");
}