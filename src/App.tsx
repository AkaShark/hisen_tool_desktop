import React, { useEffect, useState } from 'react'

// Tauri v2 API — dynamically import to handle browser context gracefully
const invokeCmd = async <T,>(cmd: string): Promise<T> => {
  const { invoke } = await import('@tauri-apps/api/core')
  return invoke<T>(cmd)
}

type NetworkIface = { name: string; received: number; transmitted: number }

type SystemInfo = {
  os_name: string | null
  hostname: string | null
  kernel_version: string | null
  os_version: string | null
  cpu_brand: string
  cpu_physical_cores: number | null
  total_memory: number
  used_memory: number
  total_swap: number
  used_swap: number
  uptime: number
  network_ifaces: NetworkIface[]
}

type AudioDevices = {
  inputs: string[]
  outputs: string[]
  default_input?: string | null
  default_output?: string | null
}

type NetTestResult = {
  external_ip?: string | null
  http_latency_ms?: number | null
  download_mbps?: number | null
  upload_mbps?: number | null
  error?: string | null
}

export default function App() {
  const [sys, setSys] = useState<SystemInfo | null>(null)
  const [audio, setAudio] = useState<AudioDevices | null>(null)
  const [cameras, setCameras] = useState<string[]>([])
  const [testing, setTesting] = useState(false)
  const [net, setNet] = useState<NetTestResult | null>(null)

  const refresh = async () => {
    try {
      const s = await invokeCmd<SystemInfo>('get_system_info')
      setSys(s)
      const a = await invokeCmd<AudioDevices>('list_audio_devices')
      setAudio(a)
      const cams = await invokeCmd<string[]>('list_cameras')
      setCameras(cams)
    } catch (e) {
      console.error('refresh error', e)
    }
  }

  const runTest = async () => {
    setTesting(true)
    setNet(null)
    try {
      const r = await invokeCmd<NetTestResult>('run_network_test')
      setNet(r)
    } catch (e: any) {
      setNet({ error: String(e) })
    } finally {
      setTesting(false)
    }
  }

  useEffect(() => {
    refresh()
  }, [])

  return (
    <div className="container">
      <h1>Hisen Desk</h1>
      <div className="actions">
        <button onClick={refresh}>刷新信息</button>
        <button onClick={runTest} disabled={testing}>{testing ? '测试中…' : '网络测试'}</button>
      </div>

      <section>
        <h2>系统信息</h2>
        {!sys && <p>加载中…</p>}
        {sys && (
          <div className="grid">
            <div><b>操作系统</b>: {sys.os_name ?? '-'}</div>
            <div><b>主机名</b>: {sys.hostname ?? '-'}</div>
            <div><b>内核版本</b>: {sys.kernel_version ?? '-'}</div>
            <div><b>OS 版本</b>: {sys.os_version ?? '-'}</div>
            <div><b>CPU</b>: {sys.cpu_brand}</div>
            <div><b>物理核心</b>: {sys.cpu_physical_cores ?? '-'}</div>
            <div><b>内存</b>: {Math.round(sys.used_memory/1024)} / {Math.round(sys.total_memory/1024)} MB</div>
            <div><b>Swap</b>: {Math.round(sys.used_swap/1024)} / {Math.round(sys.total_swap/1024)} MB</div>
            <div><b>运行时间</b>: {Math.floor(sys.uptime/3600)} 小时</div>
          </div>
        )}
      </section>

      <section>
        <h2>音频设备</h2>
        {!audio && <p>加载中…</p>}
        {audio && (
          <div className="grid two">
            <div>
              <h3>输入设备</h3>
              <ul>
                {audio.inputs.map((d, i) => <li key={i}>{d}</li>)}
              </ul>
            </div>
            <div>
              <h3>输出设备</h3>
              <ul>
                {audio.outputs.map((d, i) => <li key={i}>{d}</li>)}
              </ul>
            </div>
          </div>
        )}
        {audio && (
          <p className="muted">默认输入: {audio.default_input ?? '-'}；默认输出: {audio.default_output ?? '-'}</p>
        )}
      </section>

      <section>
        <h2>摄像头</h2>
        {!cameras.length && <p>未检测到摄像头</p>}
        {!!cameras.length && <ul>{cameras.map((c, i) => <li key={i}>{c}</li>)}</ul>}
      </section>

      <section>
        <h2>网络测试</h2>
        {!net && <p className="muted">点击“网络测试”开始测量。</p>}
        {net && (
          <div className="grid">
            <div><b>外网 IP</b>: {net.external_ip ?? '-'}</div>
            <div><b>HTTP 延迟</b>: {net.http_latency_ms != null ? `${net.http_latency_ms} ms` : '-'}</div>
            <div><b>下载速度</b>: {net.download_mbps != null ? `${net.download_mbps.toFixed(2)} Mbps` : '-'}</div>            <div><b>上传速度</b>: {net.upload_mbps != null ? `${net.upload_mbps.toFixed(2)} Mbps` : '-'}</div>            {net.error && <div className="err">错误: {net.error}</div>}
          </div>
        )}
      </section>
    </div>
  )
}
