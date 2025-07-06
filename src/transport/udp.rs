//! UDP传输实现

use crate::{Request, Response, Result, DnsError};
use crate::types::{EdnsRecord, EdnsOption, edns_option_codes};
use super::{Transport, TransportConfig};
use async_trait::async_trait;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::time::timeout;
use crate::{dns_debug, dns_info, dns_error, dns_transport};

/// UDP传输实现
#[derive(Debug)]
pub struct UdpTransport {
    config: TransportConfig,
}

impl UdpTransport {
    /// 创建新的UDP传输
    pub fn new(config: TransportConfig) -> Self {
        Self { config }
    }
    
    // 注意：移除了 default() 方法，因为它依赖兜底配置
    // 用户现在必须明确提供 TransportConfig，不能依赖隐式默认值
    // 
    // 迁移示例：
    // 旧代码: UdpTransport::default()
    // 新代码: UdpTransport::new(your_transport_config)
    
    /// Windows平台特定的socket创建
    #[cfg(windows)]
    async fn create_windows_socket(&self) -> Result<UdpSocket> {
        use std::net::SocketAddr;
        
        dns_debug!("Windows平台：开始创建UDP socket");
        
        // Windows平台尝试多种绑定策略
        let bind_addresses = [
            "0.0.0.0:0",
            "127.0.0.1:0",
            "::1:0",  // IPv6 localhost
        ];
        
        let mut last_error = None;
        
        for addr in &bind_addresses {
            dns_debug!("尝试绑定地址: {}", addr);
            match UdpSocket::bind(addr).await {
                Ok(socket) => {
                    dns_debug!("成功绑定到地址: {}", addr);
                    // 成功绑定，返回socket
                    return Ok(socket);
                }
                Err(e) => {
                    dns_debug!("绑定失败 {}: {}", addr, e);
                    last_error = Some(e);
                    // 继续尝试下一个地址
                    continue;
                }
            }
        }
        
        // 所有绑定尝试都失败
        dns_error!("所有绑定尝试都失败");
        if let Some(e) = last_error {
            Err(DnsError::Network(format!("Windows UDP socket bind failed: {}", e)))
        } else {
            Err(DnsError::Network("Windows UDP socket bind failed: unknown error".to_string()))
        }
    }
    
    /// Windows平台特定的socket配置
    #[cfg(windows)]
    async fn configure_windows_socket(&self, socket: &UdpSocket) -> Result<()> {
        // Windows平台的socket配置
        // 注意：tokio的UdpSocket没有直接的socket选项设置方法
        // 这里可以添加Windows特定的配置逻辑
        
        // 设置接收缓冲区大小（如果需要）
        // 在实际应用中，可能需要使用socket2 crate来设置更多选项
        
        Ok(())
    }
    
    /// 序列化DNS请求为字节
    pub fn serialize_request(request: &Request) -> Result<Vec<u8>> {
        dns_debug!("开始序列化DNS请求");
        dns_debug!("请求ID: {}", request.id);
        dns_debug!("查询域名: '{}'", request.query.name);
        dns_debug!("查询类型: {:?}", request.query.qtype);
        dns_debug!("客户端地址: {:?}", request.client_address);
        
        let mut buffer = Vec::with_capacity(512);
        
        // 检查是否需要EDNS记录
        let has_edns = request.client_address.is_some();
        let additional_count = if has_edns { 1u16 } else { 0u16 };
        dns_debug!("需要EDNS记录: {}, 附加记录数: {}", has_edns, additional_count);
        
        // DNS头部 (12字节)
        buffer.extend_from_slice(&request.id.to_be_bytes());
        
        // 标志位
        let mut flags = 0u16;
        if request.flags.qr { flags |= 0x8000; }
        flags |= (request.flags.opcode as u16) << 11;
        if request.flags.aa { flags |= 0x0400; }
        if request.flags.tc { flags |= 0x0200; }
        if request.flags.rd { flags |= 0x0100; }
        if request.flags.ra { flags |= 0x0080; }
        flags |= (request.flags.z as u16) << 4;
        flags |= request.flags.rcode as u16;
        buffer.extend_from_slice(&flags.to_be_bytes());
        dns_debug!("DNS头部标志位: 0x{:04X}", flags);
        
        // 问题计数
        buffer.extend_from_slice(&1u16.to_be_bytes());
        // 回答计数
        buffer.extend_from_slice(&0u16.to_be_bytes());
        // 权威计数
        buffer.extend_from_slice(&0u16.to_be_bytes());
        // 附加计数
        buffer.extend_from_slice(&additional_count.to_be_bytes());
        dns_debug!("DNS头部完成，当前缓冲区长度: {} 字节", buffer.len());
        
        // 查询部分
        let name_start_pos = buffer.len();
        Self::encode_name(&request.query.name, &mut buffer)?;
        let name_end_pos = buffer.len();
        dns_debug!("域名编码完成，占用 {} 字节 (位置 {}-{})", name_end_pos - name_start_pos, name_start_pos, name_end_pos);
        
        buffer.extend_from_slice(&u16::from(request.query.qtype).to_be_bytes());
        buffer.extend_from_slice(&u16::from(request.query.qclass).to_be_bytes());
        dns_debug!("查询类型和类别添加完成，当前缓冲区长度: {} 字节", buffer.len());
        
        // 添加EDNS记录(如果需要)
        if let Some(ref client_address) = request.client_address {
                dns_debug!("添加EDNS记录");
                Self::encode_edns_record(&mut buffer, client_address)?;
            dns_debug!("EDNS记录添加完成，最终缓冲区长度: {} 字节", buffer.len());
        }
        
        dns_debug!("DNS请求序列化完成，总长度: {} 字节", buffer.len());
        
        // 打印前64字节的十六进制内容用于调试
        let preview_len = buffer.len().min(64);
        let hex_preview: String = buffer[..preview_len].iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ");
        dns_debug!("请求数据预览 (前{}字节): {}", preview_len, hex_preview);
        
        Ok(buffer)
    }
    
    /// 编码域名
    pub fn encode_name(name: &str, buffer: &mut Vec<u8>) -> Result<()> {
        dns_debug!("编码域名: '{}'", name);
        
        if name.is_empty() || name == "." {
            dns_debug!("空域名或根域名，添加终止符");
            buffer.push(0);
            return Ok(());
        }
        
        // 移除末尾的点（如果有）
        let name = name.trim_end_matches('.');
        dns_debug!("处理后的域名: '{}'", name);
        
        for (i, label) in name.split('.').enumerate() {
            if label.is_empty() {
                dns_debug!("跳过空标签 {}", i);
                continue;
            }
            if label.len() > 63 {
                dns_debug!("标签 '{}' 长度超过63字节", label);
                return Err(DnsError::Protocol("Label too long".to_string()));
            }
            dns_debug!("添加标签 {}: '{}' (长度: {})", i, label, label.len());
            buffer.push(label.len() as u8);
            buffer.extend_from_slice(label.as_bytes());
        }
        
        dns_debug!("添加域名终止符");
        buffer.push(0);
        
        dns_debug!("域名编码完成，总长度: {} 字节", buffer.len());
        Ok(())
    }
    
    /// 序列化DNS响应
    pub fn serialize_response(response: &Response) -> Result<Vec<u8>> {
        dns_debug!("开始序列化DNS响应");
        dns_debug!("响应ID: {}", response.id);
        dns_debug!("查询数: {}, 回答数: {}, 权威数: {}, 附加数: {}", 
                  response.queries.len(), response.answers.len(), 
                  response.authorities.len(), response.additionals.len());
        
        let mut buffer = Vec::with_capacity(512);
        
        // DNS头部 (12字节)
        buffer.extend_from_slice(&response.id.to_be_bytes());
        
        // 标志位
        let mut flags = 0u16;
        if response.flags.qr { flags |= 0x8000; }
        flags |= (response.flags.opcode as u16) << 11;
        if response.flags.aa { flags |= 0x0400; }
        if response.flags.tc { flags |= 0x0200; }
        if response.flags.rd { flags |= 0x0100; }
        if response.flags.ra { flags |= 0x0080; }
        flags |= (response.flags.z as u16) << 4;
        flags |= response.flags.rcode as u16;
        buffer.extend_from_slice(&flags.to_be_bytes());
        dns_debug!("DNS头部标志位: 0x{:04X}", flags);
        
        // 计数字段
        buffer.extend_from_slice(&(response.queries.len() as u16).to_be_bytes());
        buffer.extend_from_slice(&(response.answers.len() as u16).to_be_bytes());
        buffer.extend_from_slice(&(response.authorities.len() as u16).to_be_bytes());
        buffer.extend_from_slice(&(response.additionals.len() as u16).to_be_bytes());
        dns_debug!("DNS头部完成，当前缓冲区长度: {} 字节", buffer.len());
        
        // 序列化查询部分
        for query in &response.queries {
            Self::encode_name(&query.name, &mut buffer)?;
            buffer.extend_from_slice(&u16::from(query.qtype).to_be_bytes());
            buffer.extend_from_slice(&u16::from(query.qclass).to_be_bytes());
        }
        dns_debug!("查询部分序列化完成，当前缓冲区长度: {} 字节", buffer.len());
        
        // 序列化回答部分
        for record in &response.answers {
            Self::encode_record(record, &mut buffer)?;
        }
        dns_debug!("回答部分序列化完成，当前缓冲区长度: {} 字节", buffer.len());
        
        // 序列化权威部分
        for record in &response.authorities {
            Self::encode_record(record, &mut buffer)?;
        }
        dns_debug!("权威部分序列化完成，当前缓冲区长度: {} 字节", buffer.len());
        
        // 序列化附加部分
        for record in &response.additionals {
            Self::encode_record(record, &mut buffer)?;
        }
        dns_debug!("附加部分序列化完成，最终缓冲区长度: {} 字节", buffer.len());
        
        // 打印前64字节的十六进制内容用于调试
        let preview_len = buffer.len().min(64);
        let hex_preview: String = buffer[..preview_len].iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ");
        dns_debug!("响应数据预览 (前{}字节): {}", preview_len, hex_preview);
        
        Ok(buffer)
    }
    
    /// 编码DNS记录
    pub fn encode_record(record: &crate::types::Record, buffer: &mut Vec<u8>) -> Result<()> {
        // 编码名称
        Self::encode_name(&record.name, buffer)?;
        
        // 编码类型、类别、TTL
        buffer.extend_from_slice(&u16::from(record.rtype).to_be_bytes());
        buffer.extend_from_slice(&u16::from(record.class).to_be_bytes());
        buffer.extend_from_slice(&record.ttl.to_be_bytes());
        
        // 编码数据长度和数据
        let data_bytes = Self::encode_record_data(&record.data)?;
        buffer.extend_from_slice(&(data_bytes.len() as u16).to_be_bytes());
        buffer.extend_from_slice(&data_bytes);
        
        Ok(())
    }
    
    /// 编码记录数据
    pub fn encode_record_data(data: &crate::types::RecordData) -> Result<Vec<u8>> {
        use crate::types::RecordData;
        
        match data {
            RecordData::A(ip) => Ok(ip.octets().to_vec()),
            RecordData::AAAA(ip) => Ok(ip.octets().to_vec()),
            RecordData::CNAME(name) | RecordData::NS(name) | RecordData::PTR(name) => {
                let mut buffer = Vec::new();
                Self::encode_name(name, &mut buffer)?;
                Ok(buffer)
            },
            RecordData::MX { priority, exchange } => {
                let mut buffer = Vec::new();
                buffer.extend_from_slice(&priority.to_be_bytes());
                Self::encode_name(exchange, &mut buffer)?;
                Ok(buffer)
            },
            RecordData::TXT(texts) => {
                let mut buffer = Vec::new();
                for text in texts {
                    if text.len() > 255 {
                        return Err(DnsError::Protocol("TXT record too long".to_string()));
                    }
                    buffer.push(text.len() as u8);
                    buffer.extend_from_slice(text.as_bytes());
                }
                Ok(buffer)
            },
            RecordData::SOA { mname, rname, serial, refresh, retry, expire, minimum } => {
                let mut buffer = Vec::new();
                Self::encode_name(mname, &mut buffer)?;
                Self::encode_name(rname, &mut buffer)?;
                buffer.extend_from_slice(&serial.to_be_bytes());
                buffer.extend_from_slice(&refresh.to_be_bytes());
                buffer.extend_from_slice(&retry.to_be_bytes());
                buffer.extend_from_slice(&expire.to_be_bytes());
                buffer.extend_from_slice(&minimum.to_be_bytes());
                Ok(buffer)
            },
            RecordData::SRV { priority, weight, port, target } => {
                let mut buffer = Vec::new();
                buffer.extend_from_slice(&priority.to_be_bytes());
                buffer.extend_from_slice(&weight.to_be_bytes());
                buffer.extend_from_slice(&port.to_be_bytes());
                Self::encode_name(target, &mut buffer)?;
                Ok(buffer)
            },
            RecordData::Unknown(data) => Ok(data.clone()),
        }
    }
    
    /// 反序列化DNS请求
    pub fn deserialize_request(data: &[u8]) -> Result<Request> {
        if data.len() < 12 {
            return Err(DnsError::Protocol("Request too short".to_string()));
        }
        
        let id = u16::from_be_bytes([data[0], data[1]]);
        let flags_raw = u16::from_be_bytes([data[2], data[3]]);
        
        let flags = crate::types::Flags {
            qr: (flags_raw & 0x8000) != 0,
            opcode: ((flags_raw >> 11) & 0x0F) as u8,
            aa: (flags_raw & 0x0400) != 0,
            tc: (flags_raw & 0x0200) != 0,
            rd: (flags_raw & 0x0100) != 0,
            ra: (flags_raw & 0x0080) != 0,
            z: ((flags_raw >> 4) & 0x07) as u8,
            rcode: (flags_raw & 0x0F) as u8,
        };
        
        let qdcount = u16::from_be_bytes([data[4], data[5]]);
        
        if qdcount != 1 {
            return Err(DnsError::Protocol("Request must have exactly one query".to_string()));
        }
        
        let mut offset = 12;
        
        // 解析查询部分
        let (query, _) = Self::parse_query(data, offset)?;
        
        // 检查是否有EDNS记录
        let client_address = None; // 简化处理，暂不解析EDNS
        
        Ok(Request {
            id,
            flags,
            query,
            client_address,
        })
    }
    
    /// 反序列化DNS响应
    pub fn deserialize_response(data: &[u8]) -> Result<Response> {
        if data.len() < 12 {
            return Err(DnsError::Protocol("Response too short".to_string()));
        }
        
        let id = u16::from_be_bytes([data[0], data[1]]);
        let flags_raw = u16::from_be_bytes([data[2], data[3]]);
        
        let flags = crate::types::Flags {
            qr: (flags_raw & 0x8000) != 0,
            opcode: ((flags_raw >> 11) & 0x0F) as u8,
            aa: (flags_raw & 0x0400) != 0,
            tc: (flags_raw & 0x0200) != 0,
            rd: (flags_raw & 0x0100) != 0,
            ra: (flags_raw & 0x0080) != 0,
            z: ((flags_raw >> 4) & 0x07) as u8,
            rcode: (flags_raw & 0x0F) as u8,
        };
        
        let qdcount = u16::from_be_bytes([data[4], data[5]]);
        let ancount = u16::from_be_bytes([data[6], data[7]]);
        let nscount = u16::from_be_bytes([data[8], data[9]]);
        let arcount = u16::from_be_bytes([data[10], data[11]]);
        
        let mut offset = 12;
        let mut queries = Vec::new();
        let mut answers = Vec::new();
        let mut authorities = Vec::new();
        let mut additionals = Vec::new();
        
        // 解析查询部分
        for _ in 0..qdcount {
            let (query, new_offset) = Self::parse_query(data, offset)?;
            queries.push(query);
            offset = new_offset;
        }
        
        // 解析回答部分
        for _ in 0..ancount {
            let (record, new_offset) = Self::parse_record(data, offset)?;
            answers.push(record);
            offset = new_offset;
        }
        
        // 解析权威部分
        for _ in 0..nscount {
            let (record, new_offset) = Self::parse_record(data, offset)?;
            authorities.push(record);
            offset = new_offset;
        }
        
        // 解析附加部分
        for _ in 0..arcount {
            let (record, new_offset) = Self::parse_record(data, offset)?;
            additionals.push(record);
            offset = new_offset;
        }
        
        Ok(Response {
            id,
            flags,
            queries,
            answers,
            authorities,
            additionals,
        })
    }
    
    /// 解析查询记录
    pub fn parse_query(data: &[u8], offset: usize) -> Result<(crate::types::Query, usize)> {
        let (name, mut offset) = Self::parse_name(data, offset)?;
        
        if offset + 4 > data.len() {
            return Err(DnsError::Protocol("Invalid query format".to_string()));
        }
        
        let qtype = u16::from_be_bytes([data[offset], data[offset + 1]]).into();
        let qclass = u16::from_be_bytes([data[offset + 2], data[offset + 3]]).into();
        offset += 4;
        
        Ok((crate::types::Query { name, qtype, qclass }, offset))
    }
    
    /// 解析资源记录
    pub fn parse_record(data: &[u8], offset: usize) -> Result<(crate::types::Record, usize)> {
        let (name, mut offset) = Self::parse_name(data, offset)?;
        
        if offset + 10 > data.len() {
            return Err(DnsError::Protocol("Invalid record format".to_string()));
        }
        
        let rtype = u16::from_be_bytes([data[offset], data[offset + 1]]).into();
        let class = u16::from_be_bytes([data[offset + 2], data[offset + 3]]).into();
        let ttl = u32::from_be_bytes([data[offset + 4], data[offset + 5], data[offset + 6], data[offset + 7]]);
        let rdlength = u16::from_be_bytes([data[offset + 8], data[offset + 9]]) as usize;
        offset += 10;
        
        if offset + rdlength > data.len() {
            return Err(DnsError::Protocol("Invalid record data length".to_string()));
        }
        
        let rdata = &data[offset..offset + rdlength];
        let record_data = Self::parse_record_data(rtype, rdata, data, offset)?;
        offset += rdlength;
        
        Ok((crate::types::Record {
            name,
            rtype,
            class,
            ttl,
            data: record_data,
        }, offset))
    }
    
    /// 解析域名
    pub fn parse_name(data: &[u8], mut offset: usize) -> Result<(String, usize)> {
        dns_debug!("开始解析域名，起始偏移: {}, 数据长度: {}", offset, data.len());
        
        let mut name = String::new();
        let mut jumped = false;
        let mut jump_offset = 0;
        let mut loop_count = 0;
        const MAX_LOOPS: usize = 100; // 防止无限循环
        
        loop {
            loop_count += 1;
            if loop_count > MAX_LOOPS {
                dns_debug!("域名解析循环次数超限，可能存在循环引用");
                return Err(DnsError::Protocol("Name parsing loop detected".to_string()));
            }
            
            if offset >= data.len() {
                dns_debug!("偏移量 {} 超出数据长度 {}", offset, data.len());
                return Err(DnsError::Protocol("Name parsing overflow".to_string()));
            }
            
            let len = data[offset];
            dns_debug!("偏移 {}: 长度字节 = 0x{:02X} ({})", offset, len, len);
            
            if len == 0 {
                dns_debug!("遇到域名终止符，解析完成");
                offset += 1;
                break;
            }
            
            if (len & 0xC0) == 0xC0 {
                // 压缩指针
                if offset + 1 >= data.len() {
                    dns_debug!("压缩指针数据不完整");
                    return Err(DnsError::Protocol("Incomplete compression pointer".to_string()));
                }
                
                let pointer = (((len & 0x3F) as usize) << 8) | (data[offset + 1] as usize);
                dns_debug!("压缩指针指向偏移: {}", pointer);
                
                if pointer >= data.len() {
                    dns_debug!("压缩指针 {} 超出数据范围 {}", pointer, data.len());
                    return Err(DnsError::Protocol("Invalid compression pointer".to_string()));
                }
                
                if !jumped {
                    jump_offset = offset + 2;
                    jumped = true;
                    dns_debug!("设置跳转返回点: {}", jump_offset);
                }
                
                offset = pointer;
                continue;
            }
            
            // 普通标签
            if len > 63 {
                dns_debug!("标签长度 {} 超过63字节限制", len);
                return Err(DnsError::Protocol("Label too long".to_string()));
            }
            
            offset += 1;
            if offset + len as usize > data.len() {
                dns_debug!("标签数据超出范围: 偏移{}+长度{} > 数据长度{}", offset, len, data.len());
                return Err(DnsError::Protocol("Name label overflow".to_string()));
            }
            
            if !name.is_empty() {
                name.push('.');
            }
            
            let label = String::from_utf8_lossy(&data[offset..offset + len as usize]);
            dns_debug!("解析标签: '{}'", label);
            name.push_str(&label);
            offset += len as usize;
        }
        
        if jumped {
            offset = jump_offset;
            dns_debug!("恢复到跳转返回点: {}", offset);
        }
        
        dns_debug!("域名解析完成: '{}', 最终偏移: {}", name, offset);
        Ok((name, offset))
    }
    
    /// 解析记录数据
    pub fn parse_record_data(
        rtype: crate::types::RecordType,
        rdata: &[u8],
        full_data: &[u8],
        rdata_offset: usize,
    ) -> Result<crate::types::RecordData> {
        use crate::types::{RecordType, RecordData};
        use std::net::{Ipv4Addr, Ipv6Addr};
        
        match rtype {
            RecordType::A => {
                if rdata.len() != 4 {
                    return Err(DnsError::Protocol("Invalid A record length".to_string()));
                }
                Ok(RecordData::A(Ipv4Addr::new(rdata[0], rdata[1], rdata[2], rdata[3])))
            }
            RecordType::AAAA => {
                if rdata.len() != 16 {
                    return Err(DnsError::Protocol("Invalid AAAA record length".to_string()));
                }
                let mut addr = [0u8; 16];
                addr.copy_from_slice(rdata);
                Ok(RecordData::AAAA(Ipv6Addr::from(addr)))
            }
            RecordType::CNAME | RecordType::NS | RecordType::PTR => {
                // 使用正确的偏移量来解析域名
                let (name, _) = Self::parse_name(full_data, rdata_offset)?;
                match rtype {
                    RecordType::CNAME => Ok(RecordData::CNAME(name)),
                    RecordType::NS => Ok(RecordData::NS(name)),
                    RecordType::PTR => Ok(RecordData::PTR(name)),
                    _ => unreachable!(),
                }
            }
            RecordType::MX => {
                if rdata.len() < 3 {
                    return Err(DnsError::Protocol("Invalid MX record length".to_string()));
                }
                // MX记录格式: 优先级(2字节) + 交换机域名
                let priority = u16::from_be_bytes([rdata[0], rdata[1]]);
                // 解析交换机域名，从rdata_offset + 2开始
                let (exchange, _) = Self::parse_name(full_data, rdata_offset + 2)?;
                Ok(RecordData::MX { priority, exchange })
            }
            RecordType::TXT => {
                // TXT记录可能包含多个字符串，每个字符串前有长度字节
                let mut texts = Vec::new();
                let mut offset = 0;
                while offset < rdata.len() {
                    if offset >= rdata.len() {
                        break;
                    }
                    let len = rdata[offset] as usize;
                    offset += 1;
                    if offset + len > rdata.len() {
                        return Err(DnsError::Protocol("Invalid TXT record format".to_string()));
                    }
                    let text = String::from_utf8_lossy(&rdata[offset..offset + len]).to_string();
                    texts.push(text);
                    offset += len;
                }
                Ok(RecordData::TXT(texts))
            }
            _ => Ok(RecordData::Unknown(rdata.to_vec())),
        }
    }
    
    /// 编码EDNS记录
    pub fn encode_edns_record(buffer: &mut Vec<u8>, client_address: &crate::types::ClientAddress) -> Result<()> {
        // EDNS记录格式:
        // NAME: . (root, 1字节: 0x00)
        // TYPE: OPT (41, 2字节)
        // CLASS: UDP payload size (2字节)
        // TTL: Extended RCODE + Version + Flags (4字节)
        // RDLENGTH: 选项数据长度 (2字节)
        // RDATA: 选项数据
        
        // NAME: root domain (.)
        buffer.push(0x00);
        
        // TYPE: OPT (41)
        buffer.extend_from_slice(&41u16.to_be_bytes());
        
        // CLASS: UDP payload size
        buffer.extend_from_slice(&4096u16.to_be_bytes());
        
        // TTL: Extended RCODE(1) + Version(1) + DO bit + Z(2)
        buffer.push(0); // Extended RCODE
        buffer.push(0); // Version
        buffer.extend_from_slice(&0u16.to_be_bytes()); // Flags (DO=0, Z=0)
        
        // 编码Client Address选项
        let client_address_data = client_address.encode();
        let option_length = client_address_data.len() as u16;
        
        // RDLENGTH: 选项头部(4字节) + 选项数据长度
        let rdlength = 4 + option_length;
        buffer.extend_from_slice(&rdlength.to_be_bytes());
        
        // 选项代码: Client Address (8) - 修正命名，原CLIENT_SUBNET容易误导
        buffer.extend_from_slice(&edns_option_codes::CLIENT_ADDRESS.to_be_bytes());
        
        // 选项长度
        buffer.extend_from_slice(&option_length.to_be_bytes());
        
        // 选项数据
        buffer.extend_from_slice(&client_address_data);
        
        Ok(())
    }
}

#[async_trait]
impl Transport for UdpTransport {
    async fn send(&self, request: &Request) -> Result<Response> {
        dns_debug!("UDP传输开始发送请求");
        dns_debug!("目标域名: {}", request.query.name);
        dns_debug!("查询类型: {:?}", request.query.qtype);
        
        // 平台特定的socket绑定策略
        let socket = if cfg!(windows) {
            dns_debug!("使用Windows平台socket创建策略");
            // Windows平台：使用特定的绑定策略
            self.create_windows_socket().await?
        } else {
            dns_debug!("使用Unix/Linux平台socket创建策略");
            // Unix/Linux平台：使用标准绑定
            UdpSocket::bind("0.0.0.0:0").await
                .map_err(|e| DnsError::Network(format!("Failed to bind UDP socket: {}", e)))?
        };
        
        let server_addr = format!("{}:{}", self.config.server, self.config.port);
        dns_debug!("DNS服务器地址: {}", server_addr);
        
        // 平台特定的socket配置
        if cfg!(windows) {
            dns_debug!("配置Windows socket选项");
            self.configure_windows_socket(&socket).await?;
        }
        
        let request_data = Self::serialize_request(request)?;
        dns_debug!("请求数据长度: {} 字节", request_data.len());
        
        let send_result = timeout(
            self.config.timeout,
            socket.send_to(&request_data, &server_addr)
        ).await;
        
        match send_result {
            Ok(Ok(_)) => {},
            Ok(Err(e)) => {
                let error_msg = if cfg!(windows) {
                    format!("Windows UDP send failed: {} (server: {})", e, server_addr)
                } else {
                    format!("UDP send failed: {} (server: {})", e, server_addr)
                };
                return Err(DnsError::Network(error_msg));
            },
            Err(_) => return Err(DnsError::Timeout),
        }
        
        let mut buffer = [0u8; 512];
        let recv_result = timeout(
            self.config.timeout,
            socket.recv(&mut buffer)
        ).await;
        
        let len = match recv_result {
            Ok(Ok(len)) => len,
            Ok(Err(e)) => {
                let error_msg = if cfg!(windows) {
                    format!("Windows UDP recv failed: {}", e)
                } else {
                    format!("UDP recv failed: {}", e)
                };
                return Err(DnsError::Network(error_msg));
            },
            Err(_) => return Err(DnsError::Timeout),
        };
        
        dns_debug!("收到DNS响应，长度: {} 字节", len);
        
        // 打印响应数据的十六进制内容用于调试
        let preview_len = len.min(64);
        let hex_preview: String = buffer[..preview_len].iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ");
        dns_debug!("响应数据预览 (前{}字节): {}", preview_len, hex_preview);
        
        let result = Self::deserialize_response(&buffer[..len]);
        match &result {
            Ok(response) => {
                dns_debug!("DNS响应解析成功，包含 {} 个回答记录", response.answers.len());
            },
            Err(e) => {
                dns_error!("DNS响应解析失败: {}", e);
            }
        }
        
        result
    }
    
    fn transport_type(&self) -> &'static str {
        "UDP"
    }
    
    fn set_timeout(&mut self, timeout: Duration) {
        self.config.timeout = timeout;
    }
    
    fn timeout(&self) -> Duration {
        self.config.timeout
    }
}