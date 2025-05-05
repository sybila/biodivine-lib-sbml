import biodivine_lib_sbml as sbml


def test_basic():
    file = sbml.Sbml.read_path("./test-inputs/model.sbml")
    print(file)
    model = file.model()
    print(model)
    id = model.id()
    print("Initial id:", id.get())
    new_id = sbml.SId("some_id")
    print(str(new_id), "of type", type(new_id))
    id.set(new_id)
    print("Id after being set:", model.id().get())
    assert False
