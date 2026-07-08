import './globals.css'
import { Source_Sans_3 } from 'next/font/google'
import ClientAppShell from './ClientAppShell'
import { metadata } from './metadata'

export { metadata }

const sourceSans3 = Source_Sans_3({
  subsets: ['latin'],
  weight: ['400', '500', '600', '700'],
  variable: '--font-source-sans-3',
})

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="en">
      <body className={`${sourceSans3.variable} font-sans antialiased`}>
        <ClientAppShell>{children}</ClientAppShell>
      </body>
    </html>
  )
}
