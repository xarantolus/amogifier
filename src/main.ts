import './style.css'
import typescriptLogo from './typescript.svg'
import viteLogo from '/vite.svg'
import { setupCounter } from './counter.ts'
import { read_file_to_bytes } from 'amogus'

document.querySelector('#process-button')!.addEventListener('click', async () => {
  const fileSelector = document.querySelector('#file-input') as HTMLInputElement;
  const res = await read_file_to_bytes(fileSelector.files![0])

  console.log(res)
});
