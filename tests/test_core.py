import biodivine_lib_sbml as sbml


def test_basic():
    file = sbml.Sbml.read_path("./test-inputs/Mukandavire2020.xml")
    print(file)
    model_child = file.model()
    print(model_child)
    model = model_child.get()
    print(model)
    id = model.id()
    print("Initial id:", id.get())
    new_id = sbml.SId("some_id")
    print(str(new_id), "of type", type(new_id))
    id.set(new_id)
    print("Id after being set:", model.id().get())

    parameters = model.parameters().get()
    print(parameters)
    print(parameters.len())

    for i in range(parameters.len()):
        print("Parameter ID:", parameters.get(i).id().get())
    assert False
