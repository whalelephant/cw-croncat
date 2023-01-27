import * as fs from "fs"
const artifactsRoot = `${process.cwd()}/../../artifacts`

export const getChecksums = async (): Promise<any> => {
  const sums = fs.readFileSync(`${artifactsRoot}/checksums.txt`, 'utf8')
  const lines = sums.split('\n')
  const m = {}
  lines.forEach(l => {
    const a = l.split('  ')
    const k = `${a[1]}`.replace('croncat_', '').split('.')[0]
    if (a.length > 1) m[k] = a[0]
  })
  return m  
}

export const getGitHash = () => {
  return new Promise((res, rej) => {
    require('child_process').exec('git rev-parse HEAD', function (err, stdout) {
      if (err) return rej(err)
      res(stdout)
    })
  })
}