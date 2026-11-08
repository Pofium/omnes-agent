from app.models import Item


class Service:
    def compute(self) -> int:
        return self.helper()

    def helper(self) -> int:
        return 1


@app.post("/ask")
def ask_kag(
    question: str,
    limit: int = 5,
) -> dict:
    svc = Service()
    item = Item("q")
    return {"answer": svc.compute(), "item": item}
