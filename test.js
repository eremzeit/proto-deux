
function randomQuad() {
  return Math.floor(4 * Math.random());
}

function generate() {
  let slots = new Array(10);
  const size = 10000000;
  for (let i = 0; i < size; i++) {
    let x = randomQuad();
    let y = randomQuad();
    let z = randomQuad();

    let total = x^4 * y^4 * z^4;

    if (total == 0) {
      slots[0] = (slots[0] || 0) + 1;
    } else {
      let idx = Math.floor(Math.log(total));
      //console.log('i: ', idx)
      slots[idx] = (slots[idx] || 0) + 1;
    }
  }

  console.log('')
  for (let i = 0; i < slots.length; i++) {
    console.log(`bucket: ${i}`, slots[i] / size);
  }
}

generate();
