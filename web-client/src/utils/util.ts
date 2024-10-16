interface ColorInfo {
  hex: string;
  rgb: string;
  hsl: string;
}

export type ColorAnalysis = {
  dominant: ColorInfo;
  palette: ColorInfo[];
  textColors: {
    light: ColorInfo;
    dark: ColorInfo;
  };
  suggestedAccent: ColorInfo;
  contrastRatios: {
    lightOnDominant: number;
    darkOnDominant: number;
  };
  suggestedTextColor: ColorInfo;
};

export const analyzeImageColors = async (
  base64Image: string
): Promise<ColorAnalysis> => {
  return new Promise((resolve, reject) => {
    const img = new Image();

    img.onload = () => {
      const canvas = document.createElement("canvas");
      const ctx = canvas.getContext("2d");

      if (!ctx) {
        reject(new Error("Could not get canvas context"));
        return;
      }

      // Resize image for faster processing
      const maxSize = 200;
      const scale = Math.min(maxSize / img.width, maxSize / img.height);
      canvas.width = Math.floor(img.width * scale);
      canvas.height = Math.floor(img.height * scale);

      ctx.drawImage(img, 0, 0, canvas.width, canvas.height);

      const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);
      const data = imageData.data;

      const colorMap = new Map<
        string,
        { count: number; r: number; g: number; b: number }
      >();

      for (let i = 0; i < data.length; i += 4) {
        const r = data[i];
        const g = data[i + 1];
        const b = data[i + 2];
        const a = data[i + 3];

        //calculate lightness using the standard formula
        const rprime = r / 255;
        const gprime = g / 255;
        const bprime = b / 255;
        const cmax = Math.max(rprime, gprime, bprime);
        const cmin = Math.min(rprime, gprime, bprime);
        const lightness = (cmax + cmin) / 2;

        if (
          a === 0 ||
          lightness > 0.9 || // Ignore white
          lightness < 0.1 // Ignore black
        )
          continue;

        const key = `${Math.round(r / 10)},${Math.round(g / 10)},${Math.round(
          b / 10
        )}`;
        const color = colorMap.get(key) || { count: 0, r: 0, g: 0, b: 0 };
        color.count++;
        color.r += r;
        color.g += g;
        color.b += b;
        colorMap.set(key, color);
      }

      const sortedColors = Array.from(colorMap.entries())
        .sort((a, b) => b[1].count - a[1].count)
        // eslint-disable-next-line @typescript-eslint/no-unused-vars
        .map(([_, { count, r, g, b }]) => ({
          r: Math.round(r / count),
          g: Math.round(g / count),
          b: Math.round(b / count),
        }));

      const dominant = sortedColors[0];
      const palette = sortedColors
        .slice(0, 5)
        .map((color) => rgbToColorInfo(color));

      const lightText = { r: 255, g: 255, b: 255 };
      const darkText = { r: 0, g: 0, b: 0 };

      const contrastRatios = {
        lightOnDominant: getContrastRatio(dominant, lightText),
        darkOnDominant: getContrastRatio(dominant, darkText),
      };

      const suggestedAccent = getSuggestedAccent(dominant, palette);

      const analysis: ColorAnalysis = {
        dominant: rgbToColorInfo(dominant),
        palette,
        textColors: {
          light: rgbToColorInfo(lightText),
          dark: rgbToColorInfo(darkText),
        },
        suggestedAccent,
        contrastRatios,
        suggestedTextColor:
          contrastRatios.lightOnDominant > contrastRatios.darkOnDominant
            ? rgbToColorInfo(lightText)
            : rgbToColorInfo(darkText),
      };

      resolve(analysis);
    };

    img.onerror = () => {
      reject(new Error("Failed to load image"));
    };

    img.src = base64Image;
  });
};

function rgbToColorInfo(color: { r: number; g: number; b: number }): ColorInfo {
  const hex = rgbToHex(color);
  const hsl = rgbToHsl(color);
  return {
    hex,
    rgb: `rgb(${color.r}, ${color.g}, ${color.b})`,
    hsl: `hsl(${hsl.h}, ${hsl.s}%, ${hsl.l}%)`,
  };
}

function rgbToHex({ r, g, b }: { r: number; g: number; b: number }): string {
  return "#" + [r, g, b].map((x) => x.toString(16).padStart(2, "0")).join("");
}

function rgbToHsl({ r, g, b }: { r: number; g: number; b: number }): {
  h: number;
  s: number;
  l: number;
} {
  r /= 255;
  g /= 255;
  b /= 255;
  const max = Math.max(r, g, b),
    min = Math.min(r, g, b);
  let h = 0,
    s = (max + min) / 2;
  const l = (max + min) / 2;

  if (max !== min) {
    const d = max - min;
    s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
    switch (max) {
      case r:
        h = (g - b) / d + (g < b ? 6 : 0);
        break;
      case g:
        h = (b - r) / d + 2;
        break;
      case b:
        h = (r - g) / d + 4;
        break;
    }
    h /= 6;
  }

  return {
    h: Math.round(h * 360),
    s: Math.round(s * 100),
    l: Math.round(l * 100),
  };
}

function getLuminance({
  r,
  g,
  b,
}: {
  r: number;
  g: number;
  b: number;
}): number {
  const [rs, gs, bs] = [r, g, b].map((c) => {
    c /= 255;
    return c <= 0.03928 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4);
  });
  return 0.2126 * rs + 0.7152 * gs + 0.0722 * bs;
}

function getContrastRatio(
  color1: { r: number; g: number; b: number },
  color2: { r: number; g: number; b: number }
): number {
  const l1 = getLuminance(color1);
  const l2 = getLuminance(color2);
  return (Math.max(l1, l2) + 0.05) / (Math.min(l1, l2) + 0.05);
}

function getSuggestedAccent(
  dominant: { r: number; g: number; b: number },
  palette: ColorInfo[]
): ColorInfo {
  const dominantHsl = rgbToHsl(dominant);
  const accentHsl = { ...dominantHsl, h: (dominantHsl.h + 180) % 360 };

  if (accentHsl.s < 20) accentHsl.s = 40;
  if (accentHsl.l < 20) accentHsl.l = 60;
  if (accentHsl.l > 80) accentHsl.l = 40;

  return (
    palette.find((color) => {
      const hsl = rgbToHsl(hexToRgb(color.hex));
      return Math.abs(hsl.h - accentHsl.h) < 30;
    }) || rgbToColorInfo(hslToRgb(accentHsl))
  );
}

function hexToRgb(hex: string): { r: number; g: number; b: number } {
  const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
  return result
    ? {
        r: parseInt(result[1], 16),
        g: parseInt(result[2], 16),
        b: parseInt(result[3], 16),
      }
    : { r: 0, g: 0, b: 0 };
}

function hslToRgb({ h, s, l }: { h: number; s: number; l: number }): {
  r: number;
  g: number;
  b: number;
} {
  s /= 100;
  l /= 100;
  const k = (n: number) => (n + h / 30) % 12;
  const a = s * Math.min(l, 1 - l);
  const f = (n: number) =>
    l - a * Math.max(-1, Math.min(k(n) - 3, Math.min(9 - k(n), 1)));
  return {
    r: Math.round(255 * f(0)),
    g: Math.round(255 * f(8)),
    b: Math.round(255 * f(4)),
  };
}

export const feetAndInches = (feet: number, inches = 0): number => {
  const inchesOut = feet * 12 + inches;
  return inchesOut;
};
