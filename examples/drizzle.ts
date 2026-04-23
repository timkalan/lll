import { db, users } from "./schema";
import { eq } from "drizzle-orm";

async function getUser(db: typeof db, id: number) {
  return db.select().from(users).where(eq(users.id, id));
}

async function deleteUser(db: Parameters<typeof getUser>[0], id: number) {
  return db.delete(users).where(eq(users.id, id));
}
