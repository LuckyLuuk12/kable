export interface JsHeapMetrics {
  usedBytes: number;
  totalBytes: number;
  limitBytes: number;
  usedPercent: number;
}

interface ChromiumMemoryInfo {
  usedJSHeapSize: number;
  totalJSHeapSize: number;
  jsHeapSizeLimit: number;
}

export function getJsHeapMetrics(): JsHeapMetrics | null {
  const performanceWithMemory = performance as Performance & {
    memory?: ChromiumMemoryInfo;
  };

  const memory = performanceWithMemory.memory;

  if (!memory) {
    return null;
  }

  const { usedJSHeapSize, totalJSHeapSize, jsHeapSizeLimit } = memory;

  if (
    !Number.isFinite(usedJSHeapSize) ||
    !Number.isFinite(totalJSHeapSize) ||
    !Number.isFinite(jsHeapSizeLimit) ||
    jsHeapSizeLimit <= 0
  ) {
    return null;
  }

  return {
    usedBytes: usedJSHeapSize,
    totalBytes: totalJSHeapSize,
    limitBytes: jsHeapSizeLimit,
    usedPercent: (usedJSHeapSize / jsHeapSizeLimit) * 100,
  };
}