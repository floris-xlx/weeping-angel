import { redirect } from 'next/navigation';
import { withDocsBasePath } from '@/lib/site';

export default function Home() {
  redirect(withDocsBasePath());
}
