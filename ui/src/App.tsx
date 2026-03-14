import { useMemo, useState } from 'react'
import type { FormEvent } from 'react'

import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { Input } from '@/components/ui/input'

function App() {
  const [inn, setInn] = useState('')

  const isValidInn = useMemo(() => /^\d{10}$|^\d{12}$/.test(inn), [inn])

  const handleSubmit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()
  }

  const handleChange = (value: string) => {
    setInn(value.replace(/\D/g, '').slice(0, 12))
  }

  return (
    <main className="mx-auto flex min-h-screen w-full max-w-5xl flex-col justify-center px-4 py-12 sm:px-8">
      <section className="max-w-3xl">
        <p className="text-sm uppercase tracking-[0.18em] text-muted-foreground">InfraTrace</p>
        <h1 className="mt-4 text-4xl font-semibold leading-tight tracking-tight text-foreground sm:text-6xl">
          Поиск и визуализация инфраструктуры компаний по ИНН
        </h1>
        <p className="mt-5 max-w-2xl text-base leading-relaxed text-muted-foreground sm:text-lg">
          Введите ИНН, чтобы получить карточку компании, связанные домены и сетевую инфраструктуру в
          одном отчёте.
        </p>
      </section>

      <section className="mt-14 w-full">
        <Card className="w-full border-border/80 bg-card/90">
          <CardContent className="p-5 sm:p-8">
            <form className="flex flex-col gap-4 sm:flex-row" onSubmit={handleSubmit}>
              <Input
                value={inn}
                onChange={(event) => handleChange(event.target.value)}
                inputMode="numeric"
                placeholder="Введите ИНН"
                aria-label="ИНН"
                className="h-14 text-lg"
              />
              <Button type="submit" size="xl" disabled={!isValidInn} className="w-full sm:min-w-40 sm:w-auto">
                Найти
              </Button>
            </form>
            <p className="mt-4 text-sm text-muted-foreground">ИНН: 10 или 12 цифр</p>
          </CardContent>
        </Card>
      </section>
    </main>
  )
}

export default App
