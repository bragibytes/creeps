const ADMIN_USERS = new Set(
  (process.env.ADMIN_USERS ?? '').split(',').map((s) => s.trim().toLowerCase()).filter(Boolean),
);

export function isAdmin(username: string): boolean {
  return ADMIN_USERS.has(username.toLowerCase());
}