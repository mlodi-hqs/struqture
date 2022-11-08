// Copyright © 2021-2022 HQS Quantum Simulations GmbH. All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except
// in compliance with the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the
// License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
// express or implied. See the License for the specific language governing permissions and
// limitations under the License.

// use pyo3::exceptions::PyIndexError;
use pyo3::prelude::*;
use qoqo_calculator::{CalculatorComplex, CalculatorFloat};
use qoqo_calculator_pyo3::{CalculatorComplexWrapper, CalculatorFloatWrapper};
use struqture::bosons::{
    BosonHamiltonianSystem, BosonLindbladNoiseSystem, BosonLindbladOpenSystem, BosonProduct,
    HermitianBosonProduct,
};
use struqture::{ModeIndex, OpenSystem, OperateOnDensityMatrix};
use struqture_py::bosons::{
    BosonHamiltonianSystemWrapper, BosonLindbladNoiseSystemWrapper, BosonLindbladOpenSystemWrapper,
};
use test_case::test_case;

// helper functions
fn new_system(py: Python) -> &PyCell<BosonLindbladOpenSystemWrapper> {
    let system_type = py.get_type::<BosonLindbladOpenSystemWrapper>();
    system_type
        .call0()
        .unwrap()
        .cast_as::<PyCell<BosonLindbladOpenSystemWrapper>>()
        .unwrap()
}

// helper function to convert CalculatorFloat into a python object
fn convert_cf_to_pyobject(
    py: Python,
    parameter: CalculatorFloat,
) -> &PyCell<CalculatorFloatWrapper> {
    let parameter_type = py.get_type::<CalculatorFloatWrapper>();
    match parameter {
        CalculatorFloat::Float(x) => parameter_type
            .call1((x,))
            .unwrap()
            .cast_as::<PyCell<CalculatorFloatWrapper>>()
            .unwrap(),
        CalculatorFloat::Str(x) => parameter_type
            .call1((x,))
            .unwrap()
            .cast_as::<PyCell<CalculatorFloatWrapper>>()
            .unwrap(),
    }
}

/// Test number_modes and current_number_modes functions of BosonSystem
#[test]
fn test_number_modes_current() {
    pyo3::prepare_freethreaded_python();
    pyo3::Python::with_gil(|py| {
        let system = new_system(py);
        system
            .call_method1("noise_add_operator_product", (("c0a0", "c0a0"), 0.1))
            .unwrap();

        let number_system = system.call_method0("number_modes").unwrap();
        let current_system = system.call_method0("current_number_modes").unwrap();

        let comparison =
            bool::extract(number_system.call_method1("__eq__", (1_u64,)).unwrap()).unwrap();
        assert!(comparison);
        let comparison =
            bool::extract(current_system.call_method1("__eq__", (1_u64,)).unwrap()).unwrap();
        assert!(comparison);
    });
}

/// Test empty_clone function of BosonSystem
#[test]
fn test_empty_clone() {
    pyo3::prepare_freethreaded_python();
    pyo3::Python::with_gil(|py| {
        let system = new_system(py);
        let none_system = system.call_method0("empty_clone").unwrap();
        let comparison =
            bool::extract(none_system.call_method1("__eq__", (system,)).unwrap()).unwrap();
        assert!(comparison);

        let system = new_system(py);
        let some_system = system.call_method0("empty_clone").unwrap();
        let comparison =
            bool::extract(some_system.call_method1("__eq__", (system,)).unwrap()).unwrap();
        assert!(comparison);
    });
}

/// Test add_operator_product and remove functions of BosonSystem
#[test]
fn boson_system_test_add_operator_product_remove_system() {
    pyo3::prepare_freethreaded_python();
    pyo3::Python::with_gil(|py| {
        let new_system = py.get_type::<BosonLindbladOpenSystemWrapper>();
        let number_modes: Option<usize> = Some(4);
        let system = new_system
            .call1((number_modes,))
            .unwrap()
            .cast_as::<PyCell<BosonLindbladOpenSystemWrapper>>()
            .unwrap();
        system
            .call_method1("system_add_operator_product", ("c0a0", 0.1))
            .unwrap();
        system
            .call_method1("system_add_operator_product", ("c1a2", 0.2))
            .unwrap();
        system
            .call_method1("system_add_operator_product", ("a3", 0.05))
            .unwrap();

        // test access at index 0
        let comp_op = system
            .call_method0("system")
            .unwrap()
            .call_method1("get", ("c0a0",))
            .unwrap();
        let comparison = bool::extract(comp_op.call_method1("__eq__", (0.1,)).unwrap()).unwrap();
        assert!(comparison);
        // test access at index 1
        let comp_op = system
            .call_method0("system")
            .unwrap()
            .call_method1("get", ("c1a2",))
            .unwrap();
        let comparison = bool::extract(comp_op.call_method1("__eq__", (0.2,)).unwrap()).unwrap();
        assert!(comparison);
        // test access at index 3
        let comp_op = system
            .call_method0("system")
            .unwrap()
            .call_method1("get", ("a3",))
            .unwrap();
        let comparison = bool::extract(comp_op.call_method1("__eq__", (0.05,)).unwrap()).unwrap();
        assert!(comparison);

        // Get zero
        let comp_op = system
            .call_method0("system")
            .unwrap()
            .call_method1("get", ("a2",))
            .unwrap();
        let comparison = bool::extract(comp_op.call_method1("__eq__", (0.0,)).unwrap()).unwrap();
        assert!(comparison);

        // Get error
        let error = system
            .call_method0("system")
            .unwrap()
            .call_method1("get", ("j2",));
        assert!(error.is_err());

        // Try_set error 1: Key (BosonProduct) cannot be converted from string
        let error = system.call_method1("system_add_operator_product", ("j1", 0.5));
        assert!(error.is_err());

        // Try_set error 2: Value cannot be converted to CalculatorComplex
        let error = system.call_method1("system_add_operator_product", ("c1a2", vec![0.0]));
        assert!(error.is_err());

        // Try_set error 3: Number of bosons in entry exceeds number of bosons in system.
        let error = system.call_method1("system_add_operator_product", ("a5", 0.1));
        assert!(error.is_err());

        // Try_set error 4: Generic error
        let error = system.call_method1("system_add_operator_product", ("j1", 0.5));
        assert!(error.is_err());
    });
}

/// Test add_operator_product and remove functions of BosonSystem
#[test]
fn boson_system_test_add_operator_product_remove_noise() {
    pyo3::prepare_freethreaded_python();
    pyo3::Python::with_gil(|py| {
        let new_system = py.get_type::<BosonLindbladOpenSystemWrapper>();
        let number_modes: Option<usize> = Some(4);
        let system = new_system
            .call1((number_modes,))
            .unwrap()
            .cast_as::<PyCell<BosonLindbladOpenSystemWrapper>>()
            .unwrap();
        system
            .call_method1("noise_add_operator_product", (("c0a0", "c0a0"), 0.1))
            .unwrap();
        system
            .call_method1("noise_add_operator_product", (("c0a0", "c1a2"), 0.2))
            .unwrap();
        system
            .call_method1("noise_add_operator_product", (("c0a0", "a3"), 0.05))
            .unwrap();

        // test access at index 0
        let comp_op = system
            .call_method0("noise")
            .unwrap()
            .call_method1("get", (("c0a0", "c0a0"),))
            .unwrap();
        let comparison = bool::extract(comp_op.call_method1("__eq__", (0.1,)).unwrap()).unwrap();
        assert!(comparison);
        // test access at index 1
        let comp_op = system
            .call_method0("noise")
            .unwrap()
            .call_method1("get", (("c0a0", "c1a2"),))
            .unwrap();
        let comparison = bool::extract(comp_op.call_method1("__eq__", (0.2,)).unwrap()).unwrap();
        assert!(comparison);
        // test access at index 3
        let comp_op = system
            .call_method0("noise")
            .unwrap()
            .call_method1("get", (("c0a0", "a3"),))
            .unwrap();
        let comparison = bool::extract(comp_op.call_method1("__eq__", (0.05,)).unwrap()).unwrap();
        assert!(comparison);

        // Get zero
        let comp_op = system
            .call_method0("noise")
            .unwrap()
            .call_method1("get", (("c0a0", "a2"),))
            .unwrap();
        let comparison = bool::extract(comp_op.call_method1("__eq__", (0.0,)).unwrap()).unwrap();
        assert!(comparison);

        // Get error
        let error = system
            .call_method0("noise")
            .unwrap()
            .call_method1("get", (("j2", "c0a0"),));
        assert!(error.is_err());

        // Try_set error 1: Key (BosonProduct) cannot be converted from string
        let error = system.call_method1("noise_add_operator_product", (("j1", "c0a0"), 0.5));
        assert!(error.is_err());

        // Try_set error 2: Value cannot be converted to CalculatorComplex
        let error =
            system.call_method1("noise_add_operator_product", (("c0a0", "c1a2"), vec![0.0]));
        assert!(error.is_err());

        // Try_set error 3: Number of bosons in entry exceeds number of bosons in system.
        let error = system.call_method1("noise_add_operator_product", (("c0a0", "a5"), 0.1));
        assert!(error.is_err());

        // Try_set error 4: Generic error
        let error = system.call_method1("noise_add_operator_product", (("c0a0", "j1"), 0.5));
        assert!(error.is_err());
    });
}

/// Test add magic method function of BosonSystem
#[test]
fn test_neg() {
    pyo3::prepare_freethreaded_python();
    pyo3::Python::with_gil(|py| {
        let system_0 = new_system(py);
        system_0
            .call_method1("noise_add_operator_product", (("c0a0", "c0a0"), 0.1))
            .unwrap();
        let system_1 = new_system(py);
        system_1
            .call_method1("noise_add_operator_product", (("c0a0", "c0a0"), -0.1))
            .unwrap();

        let negated = system_0.call_method0("__neg__").unwrap();
        let comparison =
            bool::extract(negated.call_method1("__eq__", (system_1,)).unwrap()).unwrap();
        assert!(comparison);
    });
}

/// Test add magic method function of BosonSystem
#[test]
fn test_add() {
    pyo3::prepare_freethreaded_python();
    pyo3::Python::with_gil(|py| {
        let system_0 = new_system(py);
        system_0
            .call_method1("noise_add_operator_product", (("c0a0", "c0a0"), 0.1))
            .unwrap();
        let system_1 = new_system(py);
        system_1
            .call_method1("noise_add_operator_product", (("c1a2", "c0a0"), 0.2))
            .unwrap();
        let system_0_1 = new_system(py);
        system_0_1
            .call_method1("noise_add_operator_product", (("c0a0", "c0a0"), 0.1))
            .unwrap();
        system_0_1
            .call_method1("noise_add_operator_product", (("c1a2", "c0a0"), 0.2))
            .unwrap();

        let added = system_0.call_method1("__add__", (system_1,)).unwrap();
        let comparison =
            bool::extract(added.call_method1("__eq__", (system_0_1,)).unwrap()).unwrap();
        assert!(comparison);
    });
}

/// Test add magic method function of BosonSystem
#[test]
fn test_sub() {
    pyo3::prepare_freethreaded_python();
    pyo3::Python::with_gil(|py| {
        let system_0 = new_system(py);
        system_0
            .call_method1("noise_add_operator_product", (("c0a0", "c0a0"), 0.1))
            .unwrap();
        let system_1 = new_system(py);
        system_1
            .call_method1("noise_add_operator_product", (("c1a2", "c0a0"), 0.2))
            .unwrap();
        let system_0_1 = new_system(py);
        system_0_1
            .call_method1("noise_add_operator_product", (("c0a0", "c0a0"), 0.1))
            .unwrap();
        system_0_1
            .call_method1("noise_add_operator_product", (("c1a2", "c0a0"), -0.2))
            .unwrap();

        let added = system_0.call_method1("__sub__", (system_1,)).unwrap();
        let comparison =
            bool::extract(added.call_method1("__eq__", (system_0_1,)).unwrap()).unwrap();
        assert!(comparison);
    });
}

/// Test add magic method function of BosonSystem
#[test]
fn test_mul_cf() {
    pyo3::prepare_freethreaded_python();
    pyo3::Python::with_gil(|py| {
        let system_0 = new_system(py);
        system_0
            .call_method1("noise_add_operator_product", (("c0a0", "c0a0"), 0.1_f64))
            .unwrap();

        let system_0_1 = new_system(py);
        system_0_1
            .call_method1("noise_add_operator_product", (("c0a0", "c0a0"), 0.2))
            .unwrap();

        let added = system_0.call_method1("__mul__", (2.0,)).unwrap();
        let comparison =
            bool::extract(added.call_method1("__eq__", (system_0_1,)).unwrap()).unwrap();
        assert!(comparison);
    });
}

/// Test default function of BosonLindbladOpenSystemWrapper
#[test]
fn test_default_partialeq_debug_clone() {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let system = new_system(py);
        let mut new_sys = system
            .call_method1(
                "system_add_operator_product",
                (
                    "c0a0",
                    convert_cf_to_pyobject(py, CalculatorFloat::from(0.1)),
                ),
            )
            .unwrap();
        new_sys = new_sys
            .call_method1(
                "noise_add_operator_product",
                (
                    ("c0a0", "c0a0"),
                    convert_cf_to_pyobject(py, CalculatorFloat::from(0.1)),
                ),
            )
            .unwrap();
        let system_wrapper = new_sys.extract::<BosonLindbladOpenSystemWrapper>().unwrap();

        // PartialEq
        let helper_ne: bool = BosonLindbladOpenSystemWrapper::default() != system_wrapper;
        assert!(helper_ne);
        let helper_eq: bool =
            BosonLindbladOpenSystemWrapper::default() == BosonLindbladOpenSystemWrapper::new(None);
        assert!(helper_eq);

        // Clone
        assert_eq!(system_wrapper.clone(), system_wrapper);

        // Debug
        assert_eq!(
            format!("{:?}", BosonLindbladOpenSystemWrapper::new(None)),
            "BosonLindbladOpenSystemWrapper { internal: BosonLindbladOpenSystem { system: BosonHamiltonianSystem { number_modes: None, hamiltonian: BosonHamiltonian { internal_map: {} } }, noise: BosonLindbladNoiseSystem { number_modes: None, operator: BosonLindbladNoiseOperator { internal_map: {} } } } }"
        );

        // Number of bosons
        let comp_op = new_sys.call_method0("number_modes").unwrap();
        let comparison = bool::extract(comp_op.call_method1("__eq__", (1,)).unwrap()).unwrap();
        assert!(comparison);

        // Current number of bosons
        let comp_op = new_sys.call_method0("current_number_modes").unwrap();
        let comparison = bool::extract(comp_op.call_method1("__eq__", (1,)).unwrap()).unwrap();
        assert!(comparison);

        // System
        let comp_op = new_sys.call_method0("system").unwrap();
        let system_type = py.get_type::<BosonHamiltonianSystemWrapper>();
        let number_modes: Option<usize> = None;
        let boson_system = system_type
            .call1((number_modes,))
            .unwrap()
            .cast_as::<PyCell<BosonHamiltonianSystemWrapper>>()
            .unwrap();
        boson_system
            .call_method1(
                "add_operator_product",
                (
                    "c0a0",
                    convert_cf_to_pyobject(py, CalculatorFloat::from(0.1)),
                ),
            )
            .unwrap();
        let comparison =
            bool::extract(comp_op.call_method1("__eq__", (boson_system,)).unwrap()).unwrap();
        assert!(comparison);

        // Noise
        let comp_op = new_sys.call_method0("noise").unwrap();
        let noise_type = py.get_type::<BosonLindbladNoiseSystemWrapper>();
        let noise = noise_type
            .call0()
            .unwrap()
            .cast_as::<PyCell<BosonLindbladNoiseSystemWrapper>>()
            .unwrap();
        noise
            .call_method1(
                "add_operator_product",
                (
                    ("c0a0", "c0a0"),
                    convert_cf_to_pyobject(py, CalculatorFloat::from(0.1)),
                ),
            )
            .unwrap();
        let comparison = bool::extract(comp_op.call_method1("__eq__", (noise,)).unwrap()).unwrap();
        assert!(comparison);

        // Ungroup + group
        let comp_op_ungroup = new_sys.call_method0("ungroup").unwrap();

        let noise_type = py.get_type::<BosonLindbladNoiseSystemWrapper>();
        let noise = noise_type
            .call0()
            .unwrap()
            .cast_as::<PyCell<BosonLindbladNoiseSystemWrapper>>()
            .unwrap();
        noise
            .call_method1(
                "add_operator_product",
                (
                    ("c0a0", "c0a0"),
                    convert_cf_to_pyobject(py, CalculatorFloat::from(0.1)),
                ),
            )
            .unwrap();

        let system_type = py.get_type::<BosonHamiltonianSystemWrapper>();
        let number_modes: Option<usize> = None;
        let boson_system = system_type
            .call1((number_modes,))
            .unwrap()
            .cast_as::<PyCell<BosonHamiltonianSystemWrapper>>()
            .unwrap();
        boson_system
            .call_method1(
                "add_operator_product",
                (
                    "c0a0",
                    convert_cf_to_pyobject(py, CalculatorFloat::from(0.1)),
                ),
            )
            .unwrap();

        let comparison = bool::extract(
            comp_op_ungroup
                .call_method1("__eq__", ((system, noise),))
                .unwrap(),
        )
        .unwrap();
        assert!(comparison);
        let comp_op_group = new_system(py)
            .call_method1("group", (system, noise))
            .unwrap();
        let comparison =
            bool::extract(comp_op_group.call_method1("__eq__", (new_sys,)).unwrap()).unwrap();
        assert!(comparison);
    })
}

/// Test set_pauli_product and get_pauli_product functions of BosonLindbladOpenSystem
#[test]
fn test_set_pauli_get_pauli() {
    pyo3::prepare_freethreaded_python();
    pyo3::Python::with_gil(|py| {
        let new_system = py.get_type::<BosonLindbladOpenSystemWrapper>();
        let new_system_1 = new_system
            .call0()
            .unwrap()
            .cast_as::<PyCell<BosonLindbladOpenSystemWrapper>>()
            .unwrap();
        let mut system = new_system_1
            .call_method1(
                "system_set",
                (
                    "c0a0",
                    convert_cf_to_pyobject(py, CalculatorFloat::from(0.1)),
                ),
            )
            .unwrap();
        system = system
            .call_method1(
                "system_set",
                (
                    "c1a2",
                    convert_cf_to_pyobject(py, CalculatorFloat::from(0.2)),
                ),
            )
            .unwrap();
        system = system
            .call_method1(
                "system_set",
                (
                    "a3",
                    convert_cf_to_pyobject(py, CalculatorFloat::from(0.05)),
                ),
            )
            .unwrap();

        // test access at index 0
        let comp_op = system
            .call_method0("system")
            .unwrap()
            .call_method1("get", ("c0a0",))
            .unwrap();
        let comparison = bool::extract(
            comp_op
                .call_method1(
                    "__eq__",
                    (convert_cf_to_pyobject(py, CalculatorFloat::from(0.1)),),
                )
                .unwrap(),
        )
        .unwrap();
        assert!(comparison);
        // test access at index 1
        let comp_op = system
            .call_method0("system")
            .unwrap()
            .call_method1("get", ("c1a2",))
            .unwrap();
        let comparison = bool::extract(
            comp_op
                .call_method1(
                    "__eq__",
                    (convert_cf_to_pyobject(py, CalculatorFloat::from(0.2)),),
                )
                .unwrap(),
        )
        .unwrap();
        assert!(comparison);
        // test access at index 3
        let comp_op = system
            .call_method0("system")
            .unwrap()
            .call_method1("get", ("a3",))
            .unwrap();
        let comparison = bool::extract(
            comp_op
                .call_method1(
                    "__eq__",
                    (convert_cf_to_pyobject(py, CalculatorFloat::from(0.05)),),
                )
                .unwrap(),
        )
        .unwrap();
        assert!(comparison);

        // Get zero
        let comp_op = system
            .call_method0("system")
            .unwrap()
            .call_method1("get", ("a2",))
            .unwrap();
        let comparison = bool::extract(
            comp_op
                .call_method1(
                    "__eq__",
                    (convert_cf_to_pyobject(py, CalculatorFloat::from(0.0)),),
                )
                .unwrap(),
        )
        .unwrap();
        assert!(comparison);

        // Try_set error 1: Key (BosonProduct) cannot be converted from string
        let error = system.call_method1(
            "system_set",
            ("j1", convert_cf_to_pyobject(py, CalculatorFloat::from(0.5))),
        );
        assert!(error.is_err());

        // Try_set error 2: Value cannot be converted to CalculatorFloat
        let error = system.call_method1("system_set", ("c1a2", vec![0.0]));
        assert!(error.is_err());

        // // Try_set error 3: Generic error
        // let error = system.call_method1("system_set", ("j1", 0.5));
        // assert!(error.is_err());
    });
}

/// Test set_noise and get_noise functions of BosonLindbladOpenSystem
#[test]
fn test_set_noise_get_noise() {
    pyo3::prepare_freethreaded_python();
    pyo3::Python::with_gil(|py| {
        let new_system = py.get_type::<BosonLindbladOpenSystemWrapper>();
        let system = new_system
            .call0()
            .unwrap()
            .cast_as::<PyCell<BosonLindbladOpenSystemWrapper>>()
            .unwrap();
        system
            .call_method1(
                "noise_set",
                (
                    ("c0a0", "c1a2"),
                    convert_cf_to_pyobject(py, CalculatorFloat::from(0.1)),
                ),
            )
            .unwrap();
        system
            .call_method1(
                "noise_set",
                (
                    ("c1a2", "a3"),
                    convert_cf_to_pyobject(py, CalculatorFloat::from(0.2)),
                ),
            )
            .unwrap();
        system
            .call_method1(
                "noise_set",
                (
                    ("c0a0", "c0a0"),
                    convert_cf_to_pyobject(py, CalculatorFloat::from(0.05)),
                ),
            )
            .unwrap();

        // test access at index 0
        let comp_op = system
            .call_method0("noise")
            .unwrap()
            .call_method1("get", (("c0a0", "c1a2"),))
            .unwrap();
        let comparison = bool::extract(
            comp_op
                .call_method1(
                    "__eq__",
                    (convert_cf_to_pyobject(py, CalculatorFloat::from(0.1)),),
                )
                .unwrap(),
        )
        .unwrap();
        assert!(comparison);
        // test access at index 1
        let comp_op = system
            .call_method0("noise")
            .unwrap()
            .call_method1("get", (("c1a2", "a3"),))
            .unwrap();
        let comparison = bool::extract(
            comp_op
                .call_method1(
                    "__eq__",
                    (convert_cf_to_pyobject(py, CalculatorFloat::from(0.2)),),
                )
                .unwrap(),
        )
        .unwrap();
        assert!(comparison);
        // test access at index 3
        let comp_op = system
            .call_method0("noise")
            .unwrap()
            .call_method1("get", (("c0a0", "c0a0"),))
            .unwrap();
        let comparison = bool::extract(
            comp_op
                .call_method1(
                    "__eq__",
                    (convert_cf_to_pyobject(py, CalculatorFloat::from(0.05)),),
                )
                .unwrap(),
        )
        .unwrap();
        assert!(comparison);

        // Get zero
        let comp_op = system
            .call_method0("noise")
            .unwrap()
            .call_method1("get", (("c2a2", "c2a2"),))
            .unwrap();
        let comparison = bool::extract(
            comp_op
                .call_method1(
                    "__eq__",
                    (convert_cf_to_pyobject(py, CalculatorFloat::from(0.0)),),
                )
                .unwrap(),
        )
        .unwrap();
        assert!(comparison);

        // Get error 1a: Key left (BosonProduct) cannot be converted from string
        let error = system
            .call_method0("noise")
            .unwrap()
            .call_method1("get", (("1+c0a1", "c1a2"),));
        assert!(error.is_err()); // same error as in ADP - Ask!

        // Get error 1b: Key right (BosonProduct) cannot be converted from string
        let error = system
            .call_method0("noise")
            .unwrap()
            .call_method1("get", (("c1a2", "1+c0a1"),));
        assert!(error.is_err()); // same error as in ADP - Ask!

        // Try_set error 1a: Key left (BosonProduct) cannot be converted from string
        let error = system.call_method1(
            "set",
            (
                ("1+c0a1", "c1a2"),
                convert_cf_to_pyobject(py, CalculatorFloat::from(0.5)),
            ),
        );
        assert!(error.is_err()); // same error as in ADP - Ask!

        // Try_set error 1b: Key right (BosonProduct) cannot be converted from string
        let error = system.call_method1(
            "set",
            (
                ("c1a2", "1+c0a1"),
                convert_cf_to_pyobject(py, CalculatorFloat::from(0.5)),
            ),
        );
        assert!(error.is_err()); // same error as in ADP - Ask!

        // Try_set error 2: Value cannot be converted to CalculatorFloat
        let error = system.call_method1("set", (("c0a0", "c0a0"), vec![0.0]));
        assert!(error.is_err());

        // // Try_set error 3: Generic error
        // let error = system.call_method1("set", ("j1", 0.5));
        // assert!(error.is_err());
    });
}

/// Test try_set_pauli_product and get_pauli_product functions of BosonLindbladOpenSystem
#[test]
fn test_try_set_pauli_get_pauli() {
    pyo3::prepare_freethreaded_python();
    pyo3::Python::with_gil(|py| {
        let new_system = py.get_type::<BosonLindbladOpenSystemWrapper>();
        let new_system_1 = new_system
            .call0()
            .unwrap()
            .cast_as::<PyCell<BosonLindbladOpenSystemWrapper>>()
            .unwrap();
        let mut system = new_system_1
            .call_method1(
                "system_set",
                (
                    "c0a0",
                    convert_cf_to_pyobject(py, CalculatorFloat::from(0.1)),
                ),
            )
            .unwrap();
        system = system
            .call_method1(
                "system_set",
                (
                    "c1a2",
                    convert_cf_to_pyobject(py, CalculatorFloat::from(0.2)),
                ),
            )
            .unwrap();
        system = system
            .call_method1(
                "system_set",
                (
                    "a3",
                    convert_cf_to_pyobject(py, CalculatorFloat::from(0.05)),
                ),
            )
            .unwrap();

        // test access at index 0
        let comp_op = system
            .call_method0("system")
            .unwrap()
            .call_method1("get", ("c0a0",))
            .unwrap();
        let comparison = bool::extract(
            comp_op
                .call_method1(
                    "__eq__",
                    (convert_cf_to_pyobject(py, CalculatorFloat::from(0.1)),),
                )
                .unwrap(),
        )
        .unwrap();
        assert!(comparison);
        // test access at index 1
        let comp_op = system
            .call_method0("system")
            .unwrap()
            .call_method1("get", ("c1a2",))
            .unwrap();
        let comparison = bool::extract(
            comp_op
                .call_method1(
                    "__eq__",
                    (convert_cf_to_pyobject(py, CalculatorFloat::from(0.2)),),
                )
                .unwrap(),
        )
        .unwrap();
        assert!(comparison);
        // test access at index 3
        let comp_op = system
            .call_method0("system")
            .unwrap()
            .call_method1("get", ("a3",))
            .unwrap();
        let comparison = bool::extract(
            comp_op
                .call_method1(
                    "__eq__",
                    (convert_cf_to_pyobject(py, CalculatorFloat::from(0.05)),),
                )
                .unwrap(),
        )
        .unwrap();
        assert!(comparison);

        // Get zero
        let comp_op = system
            .call_method0("system")
            .unwrap()
            .call_method1("get", ("a2",))
            .unwrap();
        let comparison = bool::extract(
            comp_op
                .call_method1(
                    "__eq__",
                    (convert_cf_to_pyobject(py, CalculatorFloat::from(0.0)),),
                )
                .unwrap(),
        )
        .unwrap();
        assert!(comparison);

        // Try_set error 1: Key (BosonProduct) cannot be converted from string
        let error = system.call_method1(
            "set",
            ("j1", convert_cf_to_pyobject(py, CalculatorFloat::from(0.5))),
        );
        assert!(error.is_err());

        // Try_set error 2: Value cannot be converted to CalculatorFloat
        let error = system.call_method1("set", ("c1a2", vec![0.0]));
        assert!(error.is_err());
    });
}

/// Test try_set_noise and get_noise functions of BosonLindbladOpenSystem
#[test]
fn test_try_set_noise_get_noise() {
    pyo3::prepare_freethreaded_python();
    pyo3::Python::with_gil(|py| {
        let new_system = py.get_type::<BosonLindbladOpenSystemWrapper>();
        let new_system_1 = new_system
            .call0()
            .unwrap()
            .cast_as::<PyCell<BosonLindbladOpenSystemWrapper>>()
            .unwrap();
        let mut system = new_system_1
            .call_method1(
                "noise_set",
                (
                    ("c0a0", "c1a2"),
                    convert_cf_to_pyobject(py, CalculatorFloat::from(0.1)),
                ),
            )
            .unwrap();
        system = system
            .call_method1(
                "noise_set",
                (
                    ("c1a2", "a3"),
                    convert_cf_to_pyobject(py, CalculatorFloat::from(0.2)),
                ),
            )
            .unwrap();
        system = system
            .call_method1(
                "noise_set",
                (
                    ("c0a0", "c0a0"),
                    convert_cf_to_pyobject(py, CalculatorFloat::from(0.05)),
                ),
            )
            .unwrap();

        // test access at index 0
        let comp_op = system
            .call_method0("noise")
            .unwrap()
            .call_method1("get", (("c0a0", "c1a2"),))
            .unwrap();
        let comparison = bool::extract(
            comp_op
                .call_method1(
                    "__eq__",
                    (convert_cf_to_pyobject(py, CalculatorFloat::from(0.1)),),
                )
                .unwrap(),
        )
        .unwrap();
        assert!(comparison);
        // test access at index 1
        let comp_op = system
            .call_method0("noise")
            .unwrap()
            .call_method1("get", (("c1a2", "a3"),))
            .unwrap();
        let comparison = bool::extract(
            comp_op
                .call_method1(
                    "__eq__",
                    (convert_cf_to_pyobject(py, CalculatorFloat::from(0.2)),),
                )
                .unwrap(),
        )
        .unwrap();
        assert!(comparison);
        // test access at index 3
        let comp_op = system
            .call_method0("noise")
            .unwrap()
            .call_method1("get", (("c0a0", "c0a0"),))
            .unwrap();
        let comparison = bool::extract(
            comp_op
                .call_method1(
                    "__eq__",
                    (convert_cf_to_pyobject(py, CalculatorFloat::from(0.05)),),
                )
                .unwrap(),
        )
        .unwrap();
        assert!(comparison);

        // Get zero
        let comp_op = system
            .call_method0("noise")
            .unwrap()
            .call_method1("get", (("c2a2", "c2a2"),))
            .unwrap();
        let comparison = bool::extract(
            comp_op
                .call_method1(
                    "__eq__",
                    (convert_cf_to_pyobject(py, CalculatorFloat::from(0.0)),),
                )
                .unwrap(),
        )
        .unwrap();
        assert!(comparison);

        // Try_set error 1a: Key left (BosonProduct) cannot be converted from string
        let error = system.call_method1(
            "noise_set",
            (
                ("1+c0a1", "c1a2"),
                convert_cf_to_pyobject(py, CalculatorFloat::from(0.5)),
            ),
        );
        assert!(error.is_err()); // same error as in ADP - Ask!

        // Try_set error 1b: Key right (BosonProduct) cannot be converted from string
        let error = system.call_method1(
            "noise_set",
            (
                ("c1a2", "1+c0a1"),
                convert_cf_to_pyobject(py, CalculatorFloat::from(0.5)),
            ),
        );
        assert!(error.is_err()); // same error as in ADP - Ask!

        // Try_set error 2: Value cannot be converted to CalculatorFloat
        let error = system.call_method1("noise_set", (("c0a0", "c0a0"), vec![0.0]));
        assert!(error.is_err());
    });
}

/// Test add_pauli_product and get_pauli_product functions of BosonLindbladOpenSystem
#[test]
fn test_add_pauli_get_pauli() {
    pyo3::prepare_freethreaded_python();
    pyo3::Python::with_gil(|py| {
        let new_system = py.get_type::<BosonLindbladOpenSystemWrapper>();
        let new_system_1 = new_system
            .call0()
            .unwrap()
            .cast_as::<PyCell<BosonLindbladOpenSystemWrapper>>()
            .unwrap();
        let mut system = new_system_1
            .call_method1(
                "system_add_operator_product",
                (
                    "c0a0",
                    convert_cf_to_pyobject(py, CalculatorFloat::from(0.1)),
                ),
            )
            .unwrap();
        system = system
            .call_method1(
                "system_add_operator_product",
                (
                    "c1a2",
                    convert_cf_to_pyobject(py, CalculatorFloat::from(0.2)),
                ),
            )
            .unwrap();
        system = system
            .call_method1(
                "system_add_operator_product",
                (
                    "a3",
                    convert_cf_to_pyobject(py, CalculatorFloat::from(0.05)),
                ),
            )
            .unwrap();

        // test access at index 0
        let comp_op = system
            .call_method0("system")
            .unwrap()
            .call_method1("get", ("c0a0",))
            .unwrap();
        let comparison = bool::extract(
            comp_op
                .call_method1(
                    "__eq__",
                    (convert_cf_to_pyobject(py, CalculatorFloat::from(0.1)),),
                )
                .unwrap(),
        )
        .unwrap();
        assert!(comparison);
        // test access at index 1
        let comp_op = system
            .call_method0("system")
            .unwrap()
            .call_method1("get", ("c1a2",))
            .unwrap();
        let comparison = bool::extract(
            comp_op
                .call_method1(
                    "__eq__",
                    (convert_cf_to_pyobject(py, CalculatorFloat::from(0.2)),),
                )
                .unwrap(),
        )
        .unwrap();
        assert!(comparison);
        // test access at index 3
        let comp_op = system
            .call_method0("system")
            .unwrap()
            .call_method1("get", ("a3",))
            .unwrap();
        let comparison = bool::extract(
            comp_op
                .call_method1(
                    "__eq__",
                    (convert_cf_to_pyobject(py, CalculatorFloat::from(0.05)),),
                )
                .unwrap(),
        )
        .unwrap();
        assert!(comparison);

        // Get zero
        let comp_op = system
            .call_method0("system")
            .unwrap()
            .call_method1("get", ("a2",))
            .unwrap();
        let comparison = bool::extract(
            comp_op
                .call_method1(
                    "__eq__",
                    (convert_cf_to_pyobject(py, CalculatorFloat::from(0.0)),),
                )
                .unwrap(),
        )
        .unwrap();
        assert!(comparison);

        // Get error
        let error = system
            .call_method0("system")
            .unwrap()
            .call_method1("get", ("j2",));
        assert!(error.is_err());

        // Try_set error 1: Key (BosonProduct) cannot be converted from string
        let error = system.call_method1(
            "system_add_operator_product",
            ("j1", convert_cf_to_pyobject(py, CalculatorFloat::from(0.5))),
        );
        assert!(error.is_err());

        // Try_set error 2: Value cannot be converted to CalculatorFloat
        let error = system.call_method1("system_add_operator_product", ("c1a2", vec![0.0]));
        assert!(error.is_err());

        // // Try_set error 3: Generic error
        // let error = system.call_method1("system_add_operator_product", ("j1", 0.5));
        // assert!(error.is_err());
    });
}

/// Test add_noise and get_noise functions of BosonLindbladOpenSystem
#[test]
fn test_add_noise_get_noise() {
    pyo3::prepare_freethreaded_python();
    pyo3::Python::with_gil(|py| {
        let new_system = py.get_type::<BosonLindbladOpenSystemWrapper>();
        let new_system_1 = new_system
            .call0()
            .unwrap()
            .cast_as::<PyCell<BosonLindbladOpenSystemWrapper>>()
            .unwrap();
        let mut system = new_system_1
            .call_method1(
                "noise_add_operator_product",
                (
                    ("c0a0", "c1a2"),
                    convert_cf_to_pyobject(py, CalculatorFloat::from(0.1)),
                ),
            )
            .unwrap();
        system = system
            .call_method1(
                "noise_add_operator_product",
                (
                    ("c1a2", "a3"),
                    convert_cf_to_pyobject(py, CalculatorFloat::from(0.2)),
                ),
            )
            .unwrap();
        system = system
            .call_method1(
                "noise_add_operator_product",
                (
                    ("c0a0", "c0a0"),
                    convert_cf_to_pyobject(py, CalculatorFloat::from(0.05)),
                ),
            )
            .unwrap();

        // test access at index 0
        let comp_op = system
            .call_method0("noise")
            .unwrap()
            .call_method1("get", (("c0a0", "c1a2"),))
            .unwrap();
        let comparison = bool::extract(
            comp_op
                .call_method1(
                    "__eq__",
                    (convert_cf_to_pyobject(py, CalculatorFloat::from(0.1)),),
                )
                .unwrap(),
        )
        .unwrap();
        assert!(comparison);
        // test access at index 1
        let comp_op = system
            .call_method0("noise")
            .unwrap()
            .call_method1("get", (("c1a2", "a3"),))
            .unwrap();
        let comparison = bool::extract(
            comp_op
                .call_method1(
                    "__eq__",
                    (convert_cf_to_pyobject(py, CalculatorFloat::from(0.2)),),
                )
                .unwrap(),
        )
        .unwrap();
        assert!(comparison);
        // test access at index 3
        let comp_op = system
            .call_method0("noise")
            .unwrap()
            .call_method1("get", (("c0a0", "c0a0"),))
            .unwrap();
        let comparison = bool::extract(
            comp_op
                .call_method1(
                    "__eq__",
                    (convert_cf_to_pyobject(py, CalculatorFloat::from(0.05)),),
                )
                .unwrap(),
        )
        .unwrap();
        assert!(comparison);

        // Get zero
        let comp_op = system
            .call_method0("noise")
            .unwrap()
            .call_method1("get", (("c2a2", "c2a2"),))
            .unwrap();
        let comparison = bool::extract(
            comp_op
                .call_method1(
                    "__eq__",
                    (convert_cf_to_pyobject(py, CalculatorFloat::from(0.0)),),
                )
                .unwrap(),
        )
        .unwrap();
        assert!(comparison);

        // Try_set error 1a: Key left (BosonProduct) cannot be converted from string
        let error = system.call_method1(
            "noise_add_operator_product",
            (
                ("1+c0a1", "c1a2"),
                convert_cf_to_pyobject(py, CalculatorFloat::from(0.5)),
            ),
        );
        assert!(error.is_err()); // same error as in ADP - Ask!

        // Try_set error 1b: Key right (BosonProduct) cannot be converted from string
        let error = system.call_method1(
            "noise_add_operator_product",
            (
                ("c1a2", "1+c0a1"),
                convert_cf_to_pyobject(py, CalculatorFloat::from(0.5)),
            ),
        );
        assert!(error.is_err()); // same error as in ADP - Ask!

        // Try_set error 2: Value cannot be converted to CalculatorFloat
        let error =
            system.call_method1("noise_add_operator_product", (("c0a0", "c0a0"), vec![0.0]));
        assert!(error.is_err());

        // // Try_set error 3: Generic error
        // let error = system.call_method1("noise_add_operator_product", ("j1", 0.5));
        // assert!(error.is_err());
    });
}

#[test_case(1.0,0.0;"real")]
#[test_case(0.0,1.0;"imag")]
#[test_case(0.7,0.7;"mixed")]
fn test_truncate(re: f64, im: f64) {
    pyo3::Python::with_gil(|py| {
        let system = new_system(py);
        system
            .call_method1(
                "noise_add_operator_product",
                (
                    ("c0a0", "c0a0"),
                    CalculatorComplexWrapper {
                        internal: CalculatorComplex::new(100.0 * re, 100.0 * im),
                    },
                ),
            )
            .unwrap();
        system
            .call_method1(
                "system_add_operator_product",
                (
                    "a1",
                    CalculatorFloatWrapper {
                        internal: CalculatorFloat::from(10.0 * re),
                    },
                ),
            )
            .unwrap();
        system
            .call_method1(
                "noise_add_operator_product",
                (
                    ("c0a0", "c2a2"),
                    CalculatorComplexWrapper {
                        internal: CalculatorComplex::new(re, im),
                    },
                ),
            )
            .unwrap();
        system
            .call_method1(
                "system_add_operator_product",
                (
                    "c0c1a0a2",
                    CalculatorFloatWrapper {
                        internal: CalculatorFloat::from("test"),
                    },
                ),
            )
            .unwrap();

        let test_system1 = new_system(py);
        test_system1
            .call_method1(
                "noise_add_operator_product",
                (
                    ("c0a0", "c0a0"),
                    CalculatorComplexWrapper {
                        internal: CalculatorComplex::new(100.0 * re, 100.0 * im),
                    },
                ),
            )
            .unwrap();
        test_system1
            .call_method1(
                "system_add_operator_product",
                (
                    "a1",
                    CalculatorFloatWrapper {
                        internal: CalculatorFloat::from(10.0 * re),
                    },
                ),
            )
            .unwrap();
        test_system1
            .call_method1(
                "system_add_operator_product",
                (
                    "c0c1a0a2",
                    CalculatorFloatWrapper {
                        internal: CalculatorFloat::from("test"),
                    },
                ),
            )
            .unwrap();

        let test_system2 = new_system(py);
        test_system2
            .call_method1(
                "noise_add_operator_product",
                (
                    ("c0a0", "c0a0"),
                    CalculatorComplexWrapper {
                        internal: CalculatorComplex::new(100.0 * re, 100.0 * im),
                    },
                ),
            )
            .unwrap();
        test_system2
            .call_method1(
                "system_add_operator_product",
                (
                    "c0c1a0a2",
                    CalculatorFloatWrapper {
                        internal: CalculatorFloat::from("test"),
                    },
                ),
            )
            .unwrap();

        let comparison_system1 = system.call_method1("truncate", (5.0_f64,)).unwrap();
        let comparison = bool::extract(
            comparison_system1
                .call_method1("__eq__", (test_system1,))
                .unwrap(),
        )
        .unwrap();
        assert!(comparison);

        let comparison_system2 = system.call_method1("truncate", (50.0_f64,)).unwrap();
        let comparison = bool::extract(
            comparison_system2
                .call_method1("__eq__", (test_system2,))
                .unwrap(),
        )
        .unwrap();
        assert!(comparison);
    });
}

/// Test copy and deepcopy functions of BosonLindbladOpenSystem
#[test]
fn test_copy_deepcopy() {
    pyo3::prepare_freethreaded_python();
    pyo3::Python::with_gil(|py| {
        let new_system = new_system(py);
        let mut system = new_system
            .call_method1(
                "system_add_operator_product",
                (
                    "c0a0",
                    convert_cf_to_pyobject(py, CalculatorFloat::from(0.1)),
                ),
            )
            .unwrap();
        system = system
            .call_method1(
                "noise_add_operator_product",
                (
                    ("c0a0", "c0a0"),
                    convert_cf_to_pyobject(py, CalculatorFloat::from(0.1)),
                ),
            )
            .unwrap();

        let copy_system = system.call_method0("__copy__").unwrap();
        let deepcopy_system = system.call_method1("__deepcopy__", ("",)).unwrap();
        // let copy_deepcopy_param: &PyAny = system.clone();

        let comparison_copy =
            bool::extract(copy_system.call_method1("__eq__", (system,)).unwrap()).unwrap();
        assert!(comparison_copy);
        let comparison_deepcopy =
            bool::extract(deepcopy_system.call_method1("__eq__", (system,)).unwrap()).unwrap();
        assert!(comparison_deepcopy);
    });
}

/// Test to_bincode and from_bincode functions of BosonLindbladOpenSystem
#[test]
fn test_to_from_bincode() {
    pyo3::prepare_freethreaded_python();
    pyo3::Python::with_gil(|py| {
        let new_system_1 = new_system(py);
        let mut system = new_system_1
            .call_method1(
                "system_add_operator_product",
                (
                    "c0a0",
                    convert_cf_to_pyobject(py, CalculatorFloat::from(0.1)),
                ),
            )
            .unwrap();
        system = system
            .call_method1(
                "noise_add_operator_product",
                (
                    ("c0a0", "c0a0"),
                    convert_cf_to_pyobject(py, CalculatorFloat::from(0.1)),
                ),
            )
            .unwrap();

        let serialised = system.call_method0("to_bincode").unwrap();
        let new = new_system(py);
        let deserialised = new.call_method1("from_bincode", (serialised,)).unwrap();

        let deserialised_error =
            new.call_method1("from_bincode", (bincode::serialize("fails").unwrap(),));
        assert!(deserialised_error.is_err());

        let deserialised_error =
            new.call_method1("from_bincode", (bincode::serialize(&vec![0]).unwrap(),));
        assert!(deserialised_error.is_err());

        let deserialised_error = deserialised.call_method0("from_bincode");
        assert!(deserialised_error.is_err());

        let serialised_error = serialised.call_method0("to_bincode");
        assert!(serialised_error.is_err());

        let comparison =
            bool::extract(deserialised.call_method1("__eq__", (system,)).unwrap()).unwrap();
        assert!(comparison)
    });
}

#[test]
fn test_value_error_bincode() {
    pyo3::prepare_freethreaded_python();
    pyo3::Python::with_gil(|py| {
        let new = new_system(py);
        let deserialised_error = new.call_method1("from_bincode", ("j",));
        assert!(deserialised_error.is_err());
    });
}

/// Test to_ and from_json functions of BosonLindbladOpenSystem
#[test]
fn test_to_from_json() {
    pyo3::prepare_freethreaded_python();
    pyo3::Python::with_gil(|py| {
        let new_system_1 = new_system(py);
        let mut system = new_system_1
            .call_method1(
                "system_add_operator_product",
                (
                    "c0a0",
                    convert_cf_to_pyobject(py, CalculatorFloat::from(0.1)),
                ),
            )
            .unwrap();
        system = system
            .call_method1(
                "noise_add_operator_product",
                (
                    ("c0a0", "c0a0"),
                    convert_cf_to_pyobject(py, CalculatorFloat::from(0.1)),
                ),
            )
            .unwrap();

        let serialised = system.call_method0("to_json").unwrap();
        let new = new_system(py);
        let deserialised = new.call_method1("from_json", (serialised,)).unwrap();

        let deserialised_error =
            new.call_method1("from_json", (serde_json::to_string("fails").unwrap(),));
        assert!(deserialised_error.is_err());

        let deserialised_error =
            new.call_method1("from_json", (serde_json::to_string(&vec![0]).unwrap(),));
        assert!(deserialised_error.is_err());

        let serialised_error = serialised.call_method0("to_json");
        assert!(serialised_error.is_err());

        let deserialised_error = deserialised.call_method0("from_json");
        assert!(deserialised_error.is_err());

        let comparison =
            bool::extract(deserialised.call_method1("__eq__", (system,)).unwrap()).unwrap();
        assert!(comparison)
    });
}

/// Test the __repr__ and __format__ functions
#[test]
fn test_format_repr() {
    pyo3::prepare_freethreaded_python();
    pyo3::Python::with_gil(|py| {
        let new_system = new_system(py);
        let mut system = new_system
            .call_method1(
                "system_add_operator_product",
                (
                    "c0a0",
                    convert_cf_to_pyobject(py, CalculatorFloat::from(0.1)),
                ),
            )
            .unwrap();
        system = system
            .call_method1(
                "noise_add_operator_product",
                (
                    ("c1a1", "a1"),
                    convert_cf_to_pyobject(py, CalculatorFloat::from(0.1)),
                ),
            )
            .unwrap();
        let mut rust_system = BosonLindbladOpenSystem::group(
            BosonHamiltonianSystem::new(None),
            BosonLindbladNoiseSystem::new(None),
        )
        .unwrap();
        rust_system
            .system_mut()
            .add_operator_product(
                HermitianBosonProduct::new([0], [0]).unwrap(),
                CalculatorComplex::from(0.1),
            )
            .unwrap();
        let _ = rust_system.noise_mut().add_operator_product(
            (
                BosonProduct::new([1], [1]).unwrap(),
                BosonProduct::new([1], []).unwrap(),
            ),
            CalculatorComplex::from(0.1),
        );

        let test_string =
        "BosonLindbladOpenSystem(2){\nSystem: {\nc0a0: (1e-1 + i * 0e0),\n}\nNoise: {\n(c1a1, a1): (1e-1 + i * 0e0),\n}\n}"
            .to_string();

        let to_format = system.call_method1("__format__", ("",)).unwrap();
        let format_op: &str = <&str>::extract(to_format).unwrap();
        assert_eq!(format_op, test_string);

        let to_repr = system.call_method0("__repr__").unwrap();
        let repr_op: &str = <&str>::extract(to_repr).unwrap();
        assert_eq!(repr_op, test_string);

        let to_str = system.call_method0("__str__").unwrap();
        let str_op: &str = <&str>::extract(to_str).unwrap();
        assert_eq!(str_op, test_string);
    });
}

/// Test the __richcmp__ function
#[test]
fn test_richcmp() {
    pyo3::prepare_freethreaded_python();
    pyo3::Python::with_gil(|py| {
        let new_system_1 = new_system(py);
        let mut system_one = new_system_1
            .call_method1(
                "system_add_operator_product",
                (
                    "c0a0",
                    convert_cf_to_pyobject(py, CalculatorFloat::from(0.1)),
                ),
            )
            .unwrap();
        system_one = system_one
            .call_method1(
                "noise_add_operator_product",
                (
                    ("c1a1", "a1"),
                    convert_cf_to_pyobject(py, CalculatorFloat::from(0.1)),
                ),
            )
            .unwrap();
        let new_system_1 = new_system(py);
        let mut system_two = new_system_1
            .call_method1(
                "system_add_operator_product",
                (
                    "c1a1",
                    convert_cf_to_pyobject(py, CalculatorFloat::from(0.1)),
                ),
            )
            .unwrap();
        system_two = system_two
            .call_method1(
                "noise_add_operator_product",
                (
                    ("c0a0", "a1"),
                    convert_cf_to_pyobject(py, CalculatorFloat::from(0.1)),
                ),
            )
            .unwrap();

        let comparison =
            bool::extract(system_one.call_method1("__eq__", (system_two,)).unwrap()).unwrap();
        assert!(!comparison);
        let comparison =
            bool::extract(system_one.call_method1("__eq__", ("c0a0",)).unwrap()).unwrap();
        assert!(!comparison);

        let comparison =
            bool::extract(system_one.call_method1("__ne__", (system_two,)).unwrap()).unwrap();
        assert!(comparison);
        let comparison =
            bool::extract(system_one.call_method1("__ne__", ("c0a0",)).unwrap()).unwrap();
        assert!(comparison);

        let comparison = system_one.call_method1("__ge__", ("c0a0",));
        assert!(comparison.is_err());
    });
}
