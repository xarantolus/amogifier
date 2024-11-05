import './style.css'
import { convert_image } from 'amogus'

document.querySelector('#process-button')!.addEventListener('click', async () => {
  const fileSelector = document.querySelector('#file-input') as HTMLInputElement;

  const input_bytes = await readFileAsUint8Array(fileSelector.files![0]);

  const converted_images = convert_image(input_bytes);

  function showImage(image: Uint8Array, element: string) {
    const elem = document.querySelector(element) as HTMLImageElement;
    const blob = new Blob([image]);
    const url = URL.createObjectURL(blob);
    elem!.src = url;
  }

  showImage(converted_images.preview, '#preview-image');
  showImage(converted_images.full, '#full-image');
});

function readFileAsUint8Array(file: File): Promise<Uint8Array> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => {
      const binaryString = reader.result as string;
      const bytes = new Uint8Array(binaryString.length);
      for (let i = 0; i < binaryString.length; i++) {
        bytes[i] = binaryString.charCodeAt(i);
      }
      resolve(bytes);
    };
    reader.onerror = () => reject(reader.error);
    reader.readAsBinaryString(file);
  });
}
