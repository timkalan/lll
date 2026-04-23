import { drizzle } from "drizzle-orm/node-postgres";
import { users } from "./schema";
import { eq } from "drizzle-orm";

const db = drizzle(process.env.DATABASE_URL!);

async function getUser(db: ReturnType<typeof drizzle>, id: number) {
  return db.select().from(users).where(eq(users.id, id));
}

async function deleteUser(db: Parameters<typeof getUser>[0], id: number) {
  return db.delete(users).where(eq(users.id, id));
}
