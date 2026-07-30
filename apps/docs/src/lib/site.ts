export const docsBasePath = "/docs";

export const withDocsBasePath = (path = "") => {
  if (!path || path === "/") {
    return `${docsBasePath}/`;
  }
  const normalized = path.startsWith("/") ? path : `/${path}`;
  if (normalized === docsBasePath || normalized.startsWith(`${docsBasePath}/`)) {
    return normalized;
  }
  return `${docsBasePath}${normalized}`;
};
