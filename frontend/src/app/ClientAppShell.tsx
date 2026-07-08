'use client'

import dynamic from 'next/dynamic'

const AppShell = dynamic(() => import('./AppShell'), {
  ssr: false,
  loading: () => <div className="h-screen bg-gray-50" />,
})

export default function ClientAppShell({
  children,
}: {
  children: React.ReactNode
}) {
  return <AppShell>{children}</AppShell>
}
